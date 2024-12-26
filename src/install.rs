use std::{env::consts, fs, path::PathBuf};

use crate::{check::is_downloaded, download::{copy_file_to_dir, download_packages}, env_config::{Environment, ENV_CONFIG}, environment::{configure_environment, switch_version}, zip::{auto_unzip, DEFAULT_FORMAT}, ChooseEnvironment};
use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use serde_json::Value;


pub async fn choose_and_install_from(env: &Environment, install_dir: &PathBuf) -> Result<()> {
    let args = configure_environment(env);

    install_environment(env, &args, install_dir).await?;

    Ok(())
}

/// 解压并重命名目录为指定的版本目录
fn extract_to_version_dir(filename: &str, install_dir: &PathBuf, name: &str, version: &str) -> Result<()> {
    // 创建临时解压目录
    let temp_dir = install_dir.join("temp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;
    
    // 复制到临时目录并解压
    println!("正在解压到临时目录: {}", temp_dir.display());
    let filename = copy_file_to_dir(filename, temp_dir.to_str().unwrap())?;
    auto_unzip(&filename, temp_dir.to_str().unwrap());
    
    // 创建版本目录
    let version_dir = install_dir.join(format!("{}-{}", name.to_lowercase(), version));
    if version_dir.exists() {
        fs::remove_dir_all(&version_dir)?;
    }
    
    // 检查解压后的内容是否有一个主目录
    let mut entries = fs::read_dir(&temp_dir)?;
    let first_entry = entries.next();
    
    if let Some(Ok(entry)) = first_entry {
        let path = entry.path();
        if path.is_dir() && entries.next().is_none() {
            // 只有一个目录，直接重命名
            fs::rename(path, &version_dir)?;
        } else {
            // 多个文件或直接在根目录，移动所有内容
            fs::create_dir_all(&version_dir)?;
            for entry in fs::read_dir(&temp_dir)? {
                let entry = entry?;
                let path = entry.path();
                let target = version_dir.join(path.file_name().unwrap());
                fs::rename(path, target)?;
            }
        }
    }
    
    // 清理临时目录
    fs::remove_dir_all(&temp_dir)?;
    
    Ok(())
}

pub async fn install_environment(env: &Environment, args: &Value, install_dir: &PathBuf) -> Result<()> {
    let version = args.get("version").unwrap().as_str().unwrap();
    let name = env.name.as_str();
    
    let is_downloaded = is_downloaded(name, version);

    if !is_downloaded {
        println!("{}", format!("开始安装 {}: {}...",name, version).green());
        // 下载安装包
        let package_url = choose_package(env, version);
        println!("下载地址: {}", package_url);
        
        let filename = download_packages(&package_url).await?;
        println!("下载完成: {}", filename);
        
        // 创建安装目录
        let install_dir = install_dir.join(name);
        if !install_dir.exists() {
            fs::create_dir_all(&install_dir)?;
        }
        
        // 解压并重命名到版本目录
        extract_to_version_dir(&filename, &install_dir, name, version)?;
    }
    
    // 切换版本
    switch_version(env, version)?;
    
    Ok(())
}

/// 显示交互式选择菜单并安装选中的环境
pub async fn choose_and_install(install_dir: &PathBuf) -> Result<()> {
    let environments = &ENV_CONFIG.environments;

    // 设置选项
    let items: Vec<String> = environments
        .iter()
        .map(|e| format!("{} - {}", e.name, e.description))
        .collect();

    println!("使用空格键选择/取消选择，回车键确认");
    println!("↑/↓ 或 j/k 移动光标");

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("选择要安装的环境")
        .items(&items)
        .defaults(&vec![false; items.len()])
        .interact()?;

    if selections.is_empty() {
        println!("未选择任何环境");
        return Ok(());
    }

    // 配置选中的环境
    let mut args = Vec::new();
    for &index in selections.iter() {
        let env = &environments[index];
        let arg = configure_environment(env);
        args.push(arg);
    }

    // 安装配置后的环境
    for (index, arg) in args.iter().enumerate() {
        let env = &environments[index];
        install_environment(env, arg, install_dir).await?;
    }

    Ok(())
}


pub fn select_version(prompt: &str, versions: &[String], current_version: Option<String>) -> Result<(String, bool)> {

    let current_version = current_version.unwrap_or(String::new());

    let items = versions.iter().map(|v| {
        if v == &current_version {
            format!("{} - ({})", v, "当前版本".green())
        } else {
            v.clone()
        }
    }).collect::<Vec<String>>();
    
    // 默认选中当前版本否则默认选中第一个
    let default_idx = versions.iter().position(|v| v == &current_version).unwrap_or(0);

    // 标志当前版本在安装版本中的位置
    let pos = versions.iter().position(|v| v == &current_version).unwrap_or(usize::MAX);
    
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default_idx)
        .items(&items)
        .interact()
        .unwrap();

    Ok((versions[selected].clone(), selected == pos))
}

pub fn choose_version(env: &ChooseEnvironment) -> Result<()> {

    let environments = &ENV_CONFIG.environments;
    let name = env.get_name();
    
    let choose_env = environments.iter().find(|e| e.name.to_lowercase() == name.to_lowercase());

    if let Some(env) = choose_env {
        let versions = ENV_CONFIG.get_install_versions(name);
        let current_version = ENV_CONFIG.get_current_version(name);
        
        if versions.is_empty() {
            println!("未找到 {} 的版本", name);
            return Ok(());
        }
        
        let (selected_version, skip ) = select_version("选择版本", &versions, current_version)?;
        
        // 相同版本不需要切换
        if skip {
            return Ok(());
        }
        // 切换版本
        switch_version(env, &selected_version)?;
       
        Ok(())
    } else {
        return Err(anyhow!("未找到 {} 环境", name));
    }
}

pub fn choose_package(env: &Environment, version: &str) -> String {

    let url = &env.repository;

    let mut arch = consts::ARCH;

    if arch == "x86_64" {
        arch = "x64";
    }
    let os = consts::OS;
    let mut format = &env.format;
    if format.is_null() {
        format = &DEFAULT_FORMAT;
    }

    let format = if let Some(f) = format.get(os.to_lowercase()) {
        f.as_str().unwrap()
    } else {
        &DEFAULT_FORMAT.get(os.to_lowercase()).unwrap().as_str().unwrap()
    };

    let url = url.replace("%version%", version)
        .replace("%arch%", arch)
        .replace("%platform%", os)
        .replace("%format%", format);

    url
}