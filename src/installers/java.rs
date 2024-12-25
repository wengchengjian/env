use std::{fs};
use std::path::PathBuf;
use colored::*;
use crate::environment::switch_version;
use crate::{ChooseEnvironment, Result};
use crate::check::is_downloaded;
use crate::download::{download_packages, copy_file_to_dir};
use crate::path::DEFAULT_JDK_VERSION;
use crate::zip::auto_unzip;

pub async fn install_java(install_dir: &PathBuf, version: &Option<String>) -> Result<()> {
    let version = version.clone().unwrap_or(DEFAULT_JDK_VERSION.to_string());
    
    let is_downloaded = is_downloaded(&version);

    if !is_downloaded {
        println!("{}", format!("开始安装 Java {}...", version).green());
        // 下载JDK包
        let package_url = choose_java_package(&version);
        println!("下载地址: {}", package_url);
        
        let filename = download_packages(&package_url).await?;
        println!("下载完成: {}", filename);
        
        // 创建安装目录
        let install_dir = install_dir.join("Java");
        if !install_dir.exists() {
            fs::create_dir_all(&install_dir)?;
        }
        
        // 复制到指定目录并解压
        println!("正在解压到: {}", install_dir.display());
        let filename = copy_file_to_dir(&filename, install_dir.to_str().unwrap())?;
        
        auto_unzip(&filename, install_dir.to_str().unwrap());
    }
    let env = ChooseEnvironment::Java;
    // 切换版本
    switch_version(&env, &version);
    
    Ok(())
}

fn choose_java_package(version: &String) -> String {
    let first_ver = version.split(".").next().unwrap();
    
    if cfg!(target_os = "windows") {
        format!("https://download.oracle.com/java/{}/archive/jdk-{}_windows-x64_bin.zip", first_ver, version)
    } else if cfg!(target_os = "macos") {
        format!("https://download.oracle.com/java/{}/archive/jdk-{}_macos-aarch64_bin.tar.gz", first_ver, version)
    } else {
        format!("https://download.oracle.com/java/{}/archive/jdk-{}_linux-x64_bin.tar.gz", first_ver, version)
    }
}
