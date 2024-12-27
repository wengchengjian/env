#!/usr/bin/env python3
import json
import logging
import asyncio
from socket import timeout
import aiohttp
import sys
import re
from bs4 import BeautifulSoup
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from packaging.version import parse as parse_version, Version, InvalidVersion
from tenacity import retry, stop_after_attempt, wait_exponential
from crontab import CronTab
from tqdm import tqdm

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler(Path(__file__).parent / 'update_repository.log')
    ]
)
logger = logging.getLogger(__name__)


def is_zip_file(file_path: str) -> bool:
    return file_path.endswith('.zip') or file_path.endswith('.tar.gz')

class VersionFetcher:
    def __init__(self):
        self.session = None
        self.max_versions_per_platform = 5  # 每个平台最多保留的版本数
        self.max_links_to_process = 150  # 每个环境最多处理的链接数
        # 定义平台标识符
        self.os_identifiers = {
            "windows": ["windows", "win"],
            "linux": ["linux"],
            "macos": ["macos", "darwin", "osx"]
        }
        # 定义架构标识符
        self.arch_identifiers = {
            "x64": ["x64", "amd64", "x86_64"],
            "aarch64": ["aarch64", "arm64"]
        }
        self.platforms = {
            "windows-x64": {
                "alias": ["win64", "x86_64-pc-windows", "amd64", "x64"],
                "file_types": ["zip", "msi", "exe", "7z"]
            },
            "linux-x64": {
                "alias": ["linux64", "x86_64-linux", "amd64"],
                "file_types": ["tgz", "xz", "gz"]
            },
            "linux-aarch64": {
                "alias": ["linux-arm64", "aarch64-linux"],
                "file_types": ["tgz", "xz", "gz"]
            },
            "macos-x64": {
                "alias": ["darwin64", "x86_64-darwin", "amd64"],
                "file_types": ["gz", "pkg", "dmg"]
            },
            "macos-aarch64": {
                "alias": ["darwin-arm64", "aarch64-darwin", "arm64"],
                "file_types": ["gz", "pkg", "dmg"]
            }
        }

    async def __aenter__(self):
        timeout = aiohttp.ClientTimeout(total=30)
        self.session = aiohttp.ClientSession(timeout=timeout)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    @retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=4, max=10))
    async def fetch_url(self, url: str) -> Optional[str]:
        try:
            async with self.session.get(url) as response:
                if response.status == 200:
                    return await response.text()
                logger.warning(f"Failed to fetch {url}, status: {response.status}")
                return None
        except Exception as e:
            logger.error(f"Error fetching {url}: {e}")
            raise

    async def verify_download_url(self, url: str) -> bool:
        """验证下载链接是否有效"""
        try:
            async with self.session.head(url, allow_redirects=True, timeout=1) as response:
                is_valid = response.status == 200
                logger.debug(f"URL verification {'succeeded' if is_valid else 'failed'} for {url} (status: {response.status})")
                return is_valid
        except Exception as e:
            logger.debug(f"URL verification failed for {url}: {str(e)}")
            return False

    def is_valid_file(self, url: str, platform: str) -> bool:
        """检查文件类型是否对平台有效"""
        file_type = url.split('.')[-1].lower()
        logger.debug(f"Checking file type {file_type} for platform {platform}")
        if platform not in self.platforms:
            logger.debug(f"Invalid platform: {platform}")
            return False
        valid_types = self.platforms[platform]['file_types']
        is_valid = file_type in valid_types
        logger.debug(f"File type {file_type} {'is' if is_valid else 'is not'} valid for platform {platform} (valid types: {valid_types})")
        return is_valid

    def parse_version_parts(self, version: str) -> Tuple[List[int], str]:
        """解析版本号，返回 (数字部分列表, 原始版本号)
        对于复杂版本号（如 1.2.3-beta），返回数字部分 [1,2,3] 用于排序"""
        try:
            # 尝试使用 packaging.version 解析
            parsed = parse_version(version)
            if isinstance(parsed, Version):
                # 提取主版本号数字部分
                release_parts = list(parsed.release)
                if release_parts:
                    logger.debug(f"Parsed version {version} to {release_parts}")
                    return release_parts, version
        except InvalidVersion:
            logger.debug(f"Failed to parse version {version} with packaging.version, trying fallback")
            # 如果无法解析，尝试提取数字部分
            parts = []
            current_number = ''
            for char in version:
                if char.isdigit():
                    current_number += char
                elif char == '.' and current_number:
                    parts.append(int(current_number))
                    current_number = ''
            if current_number:  # 添加最后一个数字
                parts.append(int(current_number))
            if parts:
                logger.debug(f"Fallback: parsed version {version} to {parts}")
                return parts, version
        logger.debug(f"Failed to parse version {version}")
        return [], version

    def detect_platform(self, href: str) -> str:
        """检测URL对应的平台
        返回格式：{os}-{arch}，如 windows-x64, linux-aarch64"""
        href = href.lower()
        # 检查操作系统
        detected_os = None
        for os_name, os_ident in self.os_identifiers.items():
            if any(ident in href for ident in os_ident):
                detected_os = os_name
                logger.debug(f"Detected OS {os_name} from {href}")
                break

        if not detected_os:
            logger.debug(f"No OS detected from {href}")
            return None

        # 检查架构
        detected_arch = None
        for arch_name, arch_ident in self.arch_identifiers.items():
            if any(ident in href for ident in arch_ident):
                detected_arch = arch_name
                logger.debug(f"Detected architecture {arch_name} from {href}")
                break

        if not detected_arch:
            logger.debug(f"No architecture detected from {href}, using default x64")
            detected_arch = "x64"

        platform = f"{detected_os}-{detected_arch}"
        if platform in self.platforms:
            logger.debug(f"Detected platform {platform} from {href}")
            return platform
        
        logger.debug(f"Platform {platform} not in supported platforms")
        return None

    def extract_version(self, href: str, pattern: str) -> str:
        """从URL中提取版本号"""
        try:
            match = re.search(pattern, href)
            if match:
                version = match.group(1)
                logger.debug(f"Extracted version {version} from {href}")
                return version
        except Exception as e:
            logger.error(f"Failed to extract version from {href}: {e}")
        return None

    def detect_platform_by_alias(self, href: str) -> str:
        """通过别名检测平台"""
        href = href.lower()
        for platform, info in self.platforms.items():
            if any(alias in href for alias in info['alias']):
                logger.debug(f"Detected platform {platform} by alias from {href}")
                return platform
        logger.debug(f"No platform detected by alias from {href}")
        return None

    def _log_versions(self, env_name: str, versions: Dict[str, Dict[str, str]]):
        """记录环境版本信息的日志"""
        total_versions = set()
        for platform_versions in versions.values():
            total_versions.update(platform_versions.keys())
        
        sorted_versions = sorted(total_versions, reverse=True)
        display_versions = sorted_versions[:5]
        remaining = len(sorted_versions) - 5 if len(sorted_versions) > 5 else 0
        
        version_str = ", ".join(display_versions)
        if remaining > 0:
            version_str += f" ... (and {remaining} more)"
        
        logger.info(f"[{env_name.upper()}] Found {len(total_versions)} versions at {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        logger.info(f"[{env_name.upper()}] Latest versions: {version_str}")
        
        # 记录每个平台的版本数量
        for platform, platform_versions in versions.items():
            logger.info(f"[{env_name.upper()}] {platform}: {len(platform_versions)} versions available")

    async def get_java_versions(self) -> Dict[str, Dict[str, str]]:
        versions_id = [21, 17, 11, 8]
        versions = {platform: {} for platform in self.platforms}
        logger.info(f"[JAVA] Processing versions: {', '.join(map(str, versions_id))}")

        total_tasks = len(versions_id) * len(self.platforms)
        with tqdm(total=total_tasks, desc="[JAVA] Fetching versions", unit="url") as pbar:
            for version_id in versions_id:
                for platform in self.platforms:
                    try:
                        # 构建版本号
                        # if version_id > 8:
                        #     version = f"{version_id}.0.2"
                        # else:
                        #     version = f"1.{version_id}.0"

                        # 根据平台构建URL
                        if platform.startswith("windows"):
                            url = f"https://corretto.aws/downloads/latest/amazon-corretto-{version_id}-x64-windows-jdk.zip"
                        elif platform.startswith("linux"):
                            arch = "aarch64" if platform.endswith("aarch64") else "x64"
                            url = f"https://corretto.aws/downloads/latest/amazon-corretto-{version_id}-{arch}-linux-jdk.tar.gz"
                        elif platform.startswith("macos"):
                            arch = "aarch64" if platform.endswith("aarch64") else "x64"
                            url = f"https://corretto.aws/downloads/latest/amazon-corretto-{version_id}-{arch}-macos-jdk.tar.gz"
                        else:
                            pbar.update(1)
                            continue

                        # 验证下载链接
                        if self.is_valid_file(url, platform) and await self.verify_download_url(url):
                            versions[platform][str(version_id)] = url
                            logger.debug(f"[JAVA] Added version {version_id} for {platform}: {url}")
                        pbar.update(1)
                    except Exception as e:
                        logger.error(f"[JAVA] Error processing version {version_id} for {platform}: {str(e)}")
                        pbar.update(1)
                        continue

        self._log_versions('java', versions)
        return versions

    async def get_node_versions(self) -> Dict[str, Dict[str, str]]:
        """获取Node.js的版本信息
        使用固定版本和URL模板，避免爬虫抓取
        """
        versions = {platform: {} for platform in self.platforms}
        fixed_versions = ["22.12.0", "20.18.1", "18.20.5"]  # LTS versions
        logger.info(f"[NODE] Processing fixed versions: {', '.join(fixed_versions)}")

        total_tasks = len(fixed_versions) * len(self.platforms)
        with tqdm(total=total_tasks, desc="[NODE] Fetching versions", unit="url") as pbar:
            for version in fixed_versions:
                for platform in self.platforms:
                    try:
                        # 根据平台构建URL
                        if platform.startswith("windows"):
                            url = f"https://nodejs.org/dist/v{version}/node-v{version}-win-x64.zip"
                        elif platform.startswith("linux"):
                            arch = "x64" if platform.endswith("x64") else "arm64"
                            url = f"https://nodejs.org/dist/v{version}/node-v{version}-linux-{arch}.tar.xz"
                        elif platform.startswith("macos"):
                            arch = "x64" if platform.endswith("x64") else "arm64"
                            url = f"https://nodejs.org/dist/v{version}/node-v{version}-darwin-{arch}.tar.gz"
                        else:
                            pbar.update(1)
                            continue

                        # 验证下载链接
                        if self.is_valid_file(url, platform) and await self.verify_download_url(url):
                            versions[platform][version] = url
                            logger.debug(f"[NODE] Added version {version} for {platform}: {url}")
                        pbar.update(1)
                    except Exception as e:
                        logger.error(f"[NODE] Error processing version {version} for {platform}: {str(e)}")
                        pbar.update(1)
                        continue

        self._log_versions('node', versions)
        return versions

    async def get_go_versions(self) -> Dict[str, Dict[str, str]]:
        url = "https://golang.google.cn/dl/"
        logger.info(f"[GO] Fetching versions from {url}")
        html = await self.fetch_url(url)
        if not html:
            logger.error("[GO] Failed to fetch versions")
            return {}

        soup = BeautifulSoup(html, 'lxml')
        versions = {platform: {} for platform in self.platforms}
        
        # 获取所有链接
        all_links = soup.find_all('a', href=True)
        logger.debug(f"[GO] Found {len(all_links)} total links")
        
        # 筛选符合条件的链接
        filtered_links = []
        for link in all_links:
            href = link['href']
            if not href.startswith('/dl/go'):
                continue
            version = self.extract_version(href, r'go([\d.]+[A-Za-z0-9.-]*?)').strip('.')
            if not version:
                continue
            version_parts, original_version = self.parse_version_parts(version)
            if version_parts:  # 只添加有效的版本号
                filtered_links.append((version_parts, original_version, link))

        # 按版本号排序，取最新的几个版本
        filtered_links.sort(key=lambda x: x[0], reverse=True)
        links_to_process = filtered_links[:self.max_links_to_process]
        logger.info(f"[GO] Processing {len(links_to_process)} out of {len(filtered_links)} links")
        logger.debug(f"[GO] Found {len(links_to_process)} links with valid versions")
        if links_to_process:
            logger.debug(f"[GO] Sample versions: {', '.join(str(v[1]) for v in links_to_process[:5])}")

        with tqdm(total=len(links_to_process), desc="[GO] Fetching versions", unit="version") as pbar:
            for version_parts, version, link in links_to_process:
                # 检查是否所有平台都已达到版本上限
                if all(len(versions[platform]) >= self.max_versions_per_platform for platform in self.platforms):
                    logger.info("[GO] All platforms reached version limit")
                    break

                href = link['href']
                # 首先通过URL检测平台
                platform = self.detect_platform(href)
                if not platform:
                    platform = self.detect_platform_by_alias(href)
                    if not platform:
                        logger.debug(f"[GO] No platform detected for {href}")
                        pbar.update(1)
                        continue

                # 检查该平台是否已达到版本上限
                if len(versions[platform]) >= self.max_versions_per_platform:
                    logger.debug(f"[GO] Platform {platform} reached version limit")
                    pbar.update(1)
                    continue

                # 验证下载链接
                url = f"https://golang.google.cn{href}"
                if self.is_valid_file(url, platform) and await self.verify_download_url(url):
                    versions[platform][version] = url
                    logger.debug(f"[GO] Added version {version} for {platform}: {url}")
                pbar.update(1)

        self._log_versions('go', versions)
        return versions

    async def get_rust_versions(self) -> Dict[str, Dict[str, str]]:
        versions = ["stable", "beta", "nightly"]
        logger.info("[RUST] Getting standard versions: stable, beta, nightly")
        versions_dict = {platform: {} for platform in self.platforms}

        for version in versions:
            # 检查是否所有平台都已达到版本上限
            if all(len(versions_dict[platform]) >= self.max_versions_per_platform for platform in self.platforms):
                logger.info("[RUST] All platforms reached version limit")
                break

            for platform in self.platforms:
                # 检查该平台是否已达到版本上限
                if len(versions_dict[platform]) >= self.max_versions_per_platform:
                    continue

                try:
                    # 根据平台构建URL
                    if platform.startswith("windows"):
                        triple = "x86_64-pc-windows-msvc"
                        suffix = ".exe"
                    elif platform.startswith("linux"):
                        arch = "aarch64" if platform.endswith("aarch64") else "x86_64"
                        triple = f"{arch}-unknown-linux-gnu"
                        suffix = ""
                    elif platform.startswith("macos"):
                        arch = "aarch64" if platform.endswith("aarch64") else "x86_64"
                        triple = f"{arch}-apple-darwin"
                        suffix = ""

                    url = f"https://static.rust-lang.org/rustup/dist/{triple}/rustup-init{suffix}"
                    # 验证下载链接
                    if self.is_valid_file(url, platform) and await self.verify_download_url(url):
                        versions_dict[platform][version] = url
                        logger.debug(f"[RUST] Found version {version} for {platform}: {url}")
                except Exception as e:
                    logger.error(f"[RUST] Error processing version {version} for {platform}: {str(e)}")
                    continue

        self._log_versions('rust', versions_dict)
        return versions_dict

    async def get_maven_versions(self) -> Dict[str, Dict[str, str]]:
        url = "https://maven.apache.org/download.cgi"
        logger.info(f"[MAVEN] Fetching versions from {url}")
        html = await self.fetch_url(url)
        if not html:
            logger.error("[MAVEN] Failed to fetch versions")
            return {}

        soup = BeautifulSoup(html, 'lxml')
        versions = {platform: {} for platform in self.platforms}
        
        # 获取所有版本链接并排序
        all_links = []
        for link in soup.find_all('a', href=True):
            href = link['href']
            if 'apache-maven-' not in href or 'bin.' not in href:
                continue
            version = self.extract_version(href, r'apache-maven-([\d.]+?[A-Za-z0-9.-]*?)')
            if not version:
                continue
            version_parts, original_version = self.parse_version_parts(version)
            if version_parts:  # 只添加有效的版本号
                all_links.append((version_parts, original_version, link))

        # 按版本号排序，取最新的几个版本
        all_links.sort(key=lambda x: x[0], reverse=True)
        links_to_process = all_links[:self.max_links_to_process]
        logger.info(f"[MAVEN] Processing {len(links_to_process)} out of {len(all_links)} links")

        with tqdm(total=len(links_to_process), desc="[MAVEN] Fetching versions", unit="version") as pbar:
            for version_parts, version, link in links_to_process:
                # 检查是否所有平台都已达到版本上限
                if all(len(versions[platform]) >= self.max_versions_per_platform for platform in self.platforms):
                    logger.info("[MAVEN] All platforms reached version limit")
                    break

                for platform in self.platforms:
                    # 检查该平台是否已达到版本上限
                    if len(versions[platform]) >= self.max_versions_per_platform:
                        pbar.update(1)
                        continue

                    suffix = "zip" if "windows" in platform else "tar.gz"
                    url = f"https://dlcdn.apache.org/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.{suffix}"
                    if self.is_valid_file(url, platform) and await self.verify_download_url(url):
                        versions[platform][version] = url
                        logger.debug(f"[MAVEN] Found version {version} for {platform}: {url}")
                    pbar.update(1)

        self._log_versions('maven', versions)
        return versions

    async def get_gradle_versions(self) -> Dict[str, Dict[str, str]]:
        url = "https://gradle.org/releases/"
        logger.info(f"[GRADLE] Fetching versions from {url}")
        html = await self.fetch_url(url)
        if not html:
            logger.error("[GRADLE] Failed to fetch versions")
            return {}

        soup = BeautifulSoup(html, 'lxml')
        versions = {platform: {} for platform in self.platforms}
        
        # 获取所有版本链接并排序
        all_links = []
        for link in soup.find_all('a', href=True):
            href = link['href']
            if 'gradle-' not in href or '-bin.zip' not in href:
                continue
            version = self.extract_version(href, r'gradle-([\d.]+?[A-Za-z0-9.-]*?)')
            if not version:
                continue
            version_parts, original_version = self.parse_version_parts(version)
            if version_parts:  # 只添加有效的版本号
                all_links.append((version_parts, original_version, link))

        # 按版本号排序，取最新的几个版本
        all_links.sort(key=lambda x: x[0], reverse=True)
        links_to_process = all_links[:self.max_links_to_process]
        logger.info(f"[GRADLE] Processing {len(links_to_process)} out of {len(all_links)} links")

        with tqdm(total=len(links_to_process), desc="[GRADLE] Fetching versions", unit="version") as pbar:
            for version_parts, version, link in links_to_process:
                # 检查是否所有平台都已达到版本上限
                if all(len(versions[platform]) >= self.max_versions_per_platform for platform in self.platforms):
                    logger.info("[GRADLE] All platforms reached version limit")
                    break

                url = f"https://services.gradle.org/distributions/gradle-{version}-bin.zip"
                # Gradle 使用相同的zip文件格式对所有平台
                for platform in self.platforms:
                    # 检查该平台是否已达到版本上限
                    if len(versions[platform]) >= self.max_versions_per_platform:
                        pbar.update(1)
                        continue

                    if self.is_valid_file(url, platform) and await self.verify_download_url(url):
                        versions[platform][version] = url
                        logger.debug(f"[GRADLE] Found version {version} for {platform}: {url}")
                    pbar.update(1)

        self._log_versions('gradle', versions)
        return versions

class RepositoryUpdater:
    def __init__(self):
        self.repo_file = Path(__file__).parent.parent / '.env.repository.json'
        self.config_file = Path(__file__).parent.parent / '.env.config.default.json'

    async def update(self):
        async with VersionFetcher() as fetcher:
            repository = {
                "repositories": {
                    "java": await fetcher.get_java_versions(),
                    # "python": await fetcher.get_python_versions(),
                    "go": await fetcher.get_go_versions(),
                    "node": await fetcher.get_node_versions(),
                    # "rust": await fetcher.get_rust_versions(),
                    # "maven": await fetcher.get_maven_versions(),
                    # "gradle": await fetcher.get_gradle_versions()
                }
            }

            # 更新仓库配置
            with open(self.repo_file, 'w', encoding='utf-8') as f:
                json.dump(repository, f, indent=4, ensure_ascii=False)
            
            # 更新默认配置中的版本选项
            self.update_default_config(repository)
            
            logger.info(f"Repository configuration updated successfully: {self.repo_file}")

    def update_default_config(self, repository: dict):
        if not self.config_file.exists():
            return

        with open(self.config_file, 'r', encoding='utf-8') as f:
            config = json.load(f, strict=False)

        for env in config.get('environments', []):
            name = env['name']
            if name in repository['repositories']:
                # 获取该环境的所有版本
                versions = set()
                for platform_versions in repository['repositories'][name].values():
                    versions.update(platform_versions.keys())
                versions = sorted(versions, reverse=True)

                # 更新环境的版本选项
                for arg in env.get('args', []):
                    if arg['name'] == 'version':
                        arg['options'] = versions[:4]  # 保留最新的4个版本
                        if versions:
                            arg['default'] = versions[0]  # 设置最新版本为默认值

        with open(self.config_file, 'w', encoding='utf-8') as f:
            json.dump(config, f, indent=4, ensure_ascii=False)

def setup_cron():
    try:
        cron = CronTab(user=True)
        job = cron.new(command=f'{sys.executable} {Path(__file__).absolute()} --cron')
        job.setall('0 0 * * *')  # 每天午夜执行
        cron.write()
        logger.info("Cron job setup successfully")
    except Exception as e:
        logger.error(f"Failed to setup cron job: {e}")

async def main():
    updater = RepositoryUpdater()
    await updater.update()

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == '--setup-cron':
        setup_cron()
    else:
        asyncio.run(main())
