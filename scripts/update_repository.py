#!/usr/bin/env python3
from gettext import find
import json
import logging
import asyncio
import aiohttp
import sys
import re
from bs4 import BeautifulSoup
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from packaging.version import parse as parse_version, Version, InvalidVersion
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
        self.max_versions_per_platform = 5  # 每个平台最多保留的版本数
        self.max_links_to_process = 150  # 每个环境最多处理的链接数
        self.timeout = aiohttp.ClientTimeout(total=5)
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
        # 环境配置，统一版本获取策略
        self.env_configs = {
            "java": {
                "type": "fixed_versions",
                "versions": [21, 17, 11, 8],
                "url_template": {
                    "windows-x64": "https://corretto.aws/downloads/latest/amazon-corretto-{version}-x64-windows-jdk.zip",
                    "linux-x64": "https://corretto.aws/downloads/latest/amazon-corretto-{version}-x64-linux-jdk.tar.gz",
                    "linux-aarch64": "https://corretto.aws/downloads/latest/amazon-corretto-{version}-aarch64-linux-jdk.tar.gz",
                    "macos-x64": "https://corretto.aws/downloads/latest/amazon-corretto-{version}-x64-macos-jdk.tar.gz",
                    "macos-aarch64": "https://corretto.aws/downloads/latest/amazon-corretto-{version}-aarch64-macos-jdk.tar.gz"
                }
            },
            "node": {
                "type": "fixed_versions",
                "versions": ["22.12.0", "20.18.1", "18.20.5"],
                "url_template": {
                    "windows-x64": "https://nodejs.org/dist/v{version}/node-v{version}-win-x64.zip",
                    "linux-x64": "https://nodejs.org/dist/v{version}/node-v{version}-linux-x64.tar.xz",
                    "linux-aarch64": "https://nodejs.org/dist/v{version}/node-v{version}-linux-arm64.tar.xz",
                    "macos-x64": "https://nodejs.org/dist/v{version}/node-v{version}-darwin-x64.tar.gz",
                    "macos-aarch64": "https://nodejs.org/dist/v{version}/node-v{version}-darwin-arm64.tar.gz"
                }
            },
            "go": {
                "type": "web_scrape",
                "url": "https://golang.google.cn/dl/",
                "version_pattern": r'go([\d.]+[A-Za-z0-9.-]*?)',
                "download_base": "https://golang.google.cn"
            },
            "maven": {
                "type": "fixed_versions",
                "versions": ["3.9.9", "3.9.5", "3.8.8"],
                "url_template": {
                    "windows-x64": "https://dlcdn.apache.org/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.zip",
                    "linux-x64": "https://dlcdn.apache.org/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.tar.gz",
                    "linux-aarch64": "https://dlcdn.apache.org/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.tar.gz",
                    "macos-x64": "https://dlcdn.apache.org/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.tar.gz",
                    "macos-aarch64": "https://dlcdn.apache.org/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.tar.gz"
                },
            },
            "gradle": {
                "type": "fixed_versions",
                "versions": ["8.2.1", "7.5.1", "7.1.1"],
                "url_template": {
                    "windows-x64": "https://mirrors.aliyun.com/gradle/distributions/v{version}/gradle-{version}-bin.zip",
                    "linux-x64": "https://mirrors.aliyun.com/gradle/distributions/v{version}/gradle-{version}-bin.zip",
                    "linux-aarch64": "https://mirrors.aliyun.com/gradle/distributions/v{version}/gradle-{version}-bin.zip",
                    "macos-x64": "https://mirrors.aliyun.com/gradle/distributions/v{version}/gradle-{version}-bin.zip",
                    "macos-aarch64": "https://mirrors.aliyun.com/gradle/distributions/v{version}/gradle-{version}-bin.zip"
                },
            }
        }

    async def fetch_url(self, session: aiohttp.ClientSession, url: str) -> Optional[str]:
        """获取URL内容"""
        logger.debug(f"开始获取URL内容: {url}")
        try:
            async with session.get(url) as response:
                if response.status == 200:
                    content = await response.text()
                    logger.debug(f"成功获取URL内容: {url}")
                    return content
            logger.warning(f"获取URL失败: {url}, 状态码: {response.status}")
            return None
        except asyncio.TimeoutError:
            logger.error(f"获取URL超时: {url}")
            return None
        except Exception as e:
            logger.error(f"获取URL异常: {url}, 错误: {str(e)}")
            return None

    async def verify_download_url(self, session: aiohttp.ClientSession, url: str) -> bool:
        """验证下载链接是否有效"""
        logger.debug(f"开始验证下载链接: {url}")
        try:
            async with session.head(url, allow_redirects=True) as response:
                is_valid = response.status == 200
                if is_valid:
                    logger.debug(f"下载链接有效: {url}")
                else:
                    logger.warning(f"下载链接无效: {url}, 状态码: {response.status}")
                return is_valid
        except asyncio.TimeoutError:
            logger.error(f"验证下载链接超时: {url}")
            return False
        except Exception as e:
            logger.error(f"验证下载链接异常: {url}, 错误: {str(e)}")
            return False

    async def verify_version_url(self, session: aiohttp.ClientSession, url: str, install_type: str = None) -> bool:
        """验证版本下载链接是否有效"""
        logger.debug(f"验证下载链接: {url} (类型: {install_type})")
        try:
            async with session.head(url, allow_redirects=True) as response:
                if response.status == 200:
                    logger.debug(f"下载链接有效: {url}")
                    return True
                else:
                    logger.warning(f"下载链接无效: {url}, 状态码: {response.status}")
                    return False
        except asyncio.TimeoutError:
            logger.error(f"验证下载链接超时: {url}")
            return False
        except Exception as e:
            logger.error(f"验证下载链接异常: {url}, 错误: {str(e)}")
            return False

    def is_valid_file(self, url: str, platform: str) -> bool:
        """检查文件类型是否对平台有效"""
        file_type = url.split('.')[-1].lower()
        if platform not in self.platforms:
            logger.warning(f"未知平台: {platform}, URL: {url}")
            return False
        is_valid = file_type in self.platforms[platform]['file_types']
        if not is_valid:
            logger.debug(f"文件类型无效: {file_type}, 平台: {platform}, URL: {url}")
        return is_valid

    async def _get_fixed_versions(self, session: aiohttp.ClientSession, env_name: str, config: dict) -> Dict[str, Dict[str, str]]:
        """获取固定版本的下载链接"""
        logger.info(f"开始获取{env_name}的固定版本")
        versions = {platform: {} for platform in self.platforms}
        tasks = []

        for platform in self.platforms:
            if platform not in config["url_template"]:
                logger.warning(f"{env_name}缺少平台{platform}的URL模板")
                continue

            for version in config["versions"]:
                url = config["url_template"][platform].format(version=version)
                logger.debug(f"添加版本检查任务: {env_name}, 平台: {platform}, 版本: {version}, URL: {url}")
                tasks.append(self._verify_and_add_version(session, env_name, platform, str(version), url, versions))

        if not tasks:
            logger.warning(f"{env_name}没有要检查的版本")
            return versions

        logger.info(f"开始并行检查{env_name}的{len(tasks)}个版本")
        await asyncio.gather(*tasks)
        logger.info(f"完成{env_name}的固定版本检查")
        return versions

    async def _verify_and_add_version(self, session: aiohttp.ClientSession, env_name: str, platform: str, version: str, url: str, versions: dict):
        """验证并添加版本"""
        logger.debug(f"开始验证版本: {env_name}, 平台: {platform}, 版本: {version}, URL: {url}")
        if await self.verify_download_url(session, url):
            versions[platform][version] = url
            logger.info(f"找到有效版本: {env_name}, 平台: {platform}, 版本: {version}")
        else:
            logger.warning(f"版本验证失败: {env_name}, 平台: {platform}, 版本: {version}")

    async def _get_web_versions(self, session: aiohttp.ClientSession, env_name: str, config: dict) -> Dict[str, Dict[str, str]]:
        """从网页抓取版本信息"""
        logger.info(f"开始从网页获取{env_name}的版本信息: {config['url']}")
        versions = {platform: {} for platform in self.platforms}
        
        try:
            content = await self.fetch_url(session, config["url"])
            if not content:
                logger.error(f"获取{env_name}的网页内容失败")
                return versions

            logger.debug(f"开始解析{env_name}的网页内容")
            soup = BeautifulSoup(content, 'html.parser')
            links = soup.find_all('a', href=True)
            logger.debug(f"找到{len(links)}个链接")
            
            version_pattern = re.compile(config["version_pattern"])
            logger.debug(f"使用版本匹配模式: {config['version_pattern']}")
            
            # 获取所有版本号
            version_map = dict()
            for link in links:
                href = link['href']
                match = version_pattern.search(href)
                if match:
                    version = match.group(1)
                    version = version.strip('.')
                    try:
                        # 验证版本号格式
                        parse_version(version)
                        if version_map.get(version) is None:
                            version_map[version] = []
                        version_map[version].append(href)
                        logger.debug(f"找到有效版本: {version}, URL: {href}")
                    except InvalidVersion:
                        logger.warning(f"无效的版本号格式: {version}, url:{href}")
                        continue

            if len(version_map.keys()) == 0:
                logger.warning(f"{env_name}没有找到任何版本")
                return versions

            logger.info(f"共找到{len(version_map.keys())}个不同版本")
            
            # 获取每个版本的下载链接
            valid_versions = 0
            sorted_versions = sorted(version_map.items(), reverse=True, key=lambda v: parse_version(v[0]))
            all_version_num = 0
            for version, hrefs in sorted_versions:
                # 超过了限制的最大版本数量
                if all_version_num >= self.max_versions_per_platform:
                    break

                for href in hrefs:
                    # 检查每个平台的版本都有了就跳过
                    if all(versions[platform].get(version) is not None for platform in self.platforms):
                        all_version_num += 1
                        break
                    try:
                        logger.debug(f"处理版本 {version}")
                            # 原有的处理逻辑
                        platform_found = False
                        for platform in self.platforms:
                            if versions[platform].get(version) is not None:
                                continue
                            if any(ident in href.lower() for ident in self.platforms[platform]["alias"]):
                                logger.debug(f"通过别名匹配到平台 {platform}")
                                if "download_base" in config:
                                    if "filename_template" in config:
                                        url = f"{config['download_base'].format(version=version)}/{config['filename_template'].format(version=version, suffix='zip')}"
                                    else:
                                        url = config["download_base"] + href
                                else:
                                    url = href if href.startswith('http') else config["url"] + href
                                
                                logger.debug(f"生成下载链接: {url}")
                                if self.is_valid_file(url, platform) and await self.verify_version_url(session, url):
                                    versions[platform][version] = url
                                    platform_found = True
                                    logger.info(f"找到{env_name} {version}版本的{platform}平台下载包: {url}")
                                else:
                                    logger.debug(f"下载链接无效或验证失败: {url}")
                            
                            if platform_found:
                                valid_versions += 1
                                logger.debug(f"版本 {version} 找到了有效的平台下载链接")
                            else:
                                logger.debug(f"版本 {version} 未找到任何平台的下载链接")

                    except Exception as e:
                        logger.error(f"处理版本{version}时发生错误: {str(e)}", exc_info=True)
                        continue

            logger.info(f"{env_name}共找到{valid_versions}个有效版本")
            return versions

        except Exception as e:
            logger.error(f"获取{env_name}版本信息时发生错误: {str(e)}", exc_info=True)
            return versions

    def _log_versions(self, env_name: str, versions: Dict[str, Dict[str, str]]):
        """记录找到的版本信息"""
        total_versions = 0
        for platform, platform_versions in versions.items():
            version_count = len(platform_versions)
            total_versions += version_count
            if version_count > 0:
                logger.info(f"[{env_name}] 平台 {platform} 找到 {version_count} 个版本")
                try:
                    latest = max(platform_versions.keys(), key=lambda v: parse_version(v) if not v.startswith('v') else parse_version(v[1:]))
                    logger.info(f"[{env_name}] 平台 {platform} 最新版本: {latest}")
                except (ValueError, InvalidVersion) as e:
                    logger.error(f"[{env_name}] 平台 {platform} 版本解析错误: {e}")
            else:
                logger.warning(f"[{env_name}] 平台 {platform} 未找到任何版本")
        
        logger.info(f"[{env_name}] 总共找到 {total_versions} 个版本")

    async def get_versions(self, env_name: str) -> Dict[str, Dict[str, str]]:
        """获取指定环境的所有版本信息"""
        logger.info(f"开始获取环境 {env_name} 的版本信息")
        if env_name not in self.env_configs:
            logger.error(f"未知环境: {env_name}")
            return {}

        config = self.env_configs[env_name]
        try:
            async with aiohttp.ClientSession(timeout=self.timeout) as session:
                if config["type"] == "fixed_versions":
                    versions = await self._get_fixed_versions(session, env_name, config)
                elif config["type"] == "web_scrape":
                    versions = await self._get_web_versions(session, env_name, config)
                else:
                    logger.error(f"未知的版本类型: {config['type']}")
                    return {}
                
                self._log_versions(env_name, versions)
                return versions
        except Exception as e:
            logger.error(f"获取 {env_name} 版本时发生错误: {str(e)}")
            return {}

    async def __aenter__(self):
        logger.debug("进入 VersionFetcher 上下文")
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        logger.debug("退出 VersionFetcher 上下文")
        if exc_type:
            logger.error(f"VersionFetcher 上下文发生错误: {exc_type.__name__}: {str(exc_val)}")

class RepositoryUpdater:
    def __init__(self):
        self.repo_file = Path(__file__).parent.parent / '.env.repository.json'
        self.config_file = Path(__file__).parent.parent / '.env.config.default.json'

    async def update(self):
        """更新所有环境的版本信息"""
        repository = {}
        async with VersionFetcher() as fetcher:
            for env_name in fetcher.env_configs:
                logger.info(f"Fetching versions for {env_name}...")
                versions = await fetcher.get_versions(env_name)
                if versions:
                    repository[env_name] = versions

        # 保存到文件
        with open(self.repo_file, 'w', encoding='utf-8') as f:
            json.dump(repository, f, indent=2)
        logger.info(f"Repository updated and saved to {self.repo_file}")

        # 更新默认配置
        self.update_default_config(repository)

    def update_default_config(self, repository: dict):
        """更新默认配置文件"""
        if not repository:
            return

        config = {}
        if self.config_file.exists():
            with open(self.config_file, 'r', encoding='utf-8') as f:
                config = json.load(f)

        # 更新配置
        for env_name, versions in repository.items():
            if not versions:
                continue
            enviroments= config['environments']
            
            find_env = [env for env in enviroments if env['name'] == env_name]
            
            if not find_env:
                continue

            find_env = find_env[0]
            
            for arg in find_env['args']:
                if arg['name'] == 'version':
                    # 添加option version并排序
                    vers = sorted(list(list(versions.values())[0].keys()), reverse=True, key=lambda v: parse_version(v) if not v.startswith('v') else parse_version(v[1:]))

                    arg['options'] = vers
                    arg['default'] = max(vers, key = lambda v: parse_version(v) if not v.startswith('v') else parse_version(v[1:]))
        # 保存配置
        with open(self.config_file, 'w', encoding='utf-8') as f:
            json.dump(config, f, indent=2)
        logger.info(f"Default config updated and saved to {self.config_file}")

def setup_cron():
    """设置定时任务"""
    cron = CronTab(user=True)
    job = cron.new(command=f'{sys.executable} {__file__}')
    job.hour.on(0)  # 每天0点执行
    cron.write()
    logger.info("Cron job set up successfully")

async def main():
    updater = RepositoryUpdater()
    await updater.update()

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == '--setup-cron':
        setup_cron()
    else:
        asyncio.run(main())
