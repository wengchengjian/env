use std::{fmt::Display, path::PathBuf};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use crate::{env_config::EnvConfig, path::{set_persistent_env, set_persistent_path}, ChooseEnvironment, Result};

pub mod environments;


pub fn switch_version(env: &ChooseEnvironment, version: &str) {
    let ret = match env {
        ChooseEnvironment::Java => switch_jdk_version(version),
        _ => {
            println!("{} 环境不支持切换版本", env.get_name());
            Ok(())
        }
    };
    match ret {
        Ok(_) => println!("{} 切换完成!", env.get_name()),
        Err(e) => println!("{} 切换失败: {}", env.get_name(), e),
    };

    
}

pub fn switch_jdk_version(version: &str) -> Result<()> {
    let config = EnvConfig::load_deserialize();
    let install_dir = PathBuf::from(&config.install_path);
    let install_dir = install_dir.join("Java");
    // JDK目录
    let jdk_dir = install_dir.join(format!("jdk-{}", version));
    let jdk_home = jdk_dir.to_str().unwrap();
    
    // 设置环境变量
    println!("正在设置环境变量...");
    set_persistent_env("JAVA_HOME", jdk_home)?;
    println!("添加Path...");
    set_persistent_path(Some("JAVA_HOME"), "bin")?;
    
    println!("{}", "Java切换完成!".green());
    println!("JAVA_HOME: {}", jdk_home);
    println!("版本: {}", version);
    Ok(())
}

/// 配置工具函数：选择版本
pub fn select_version<'a, T>(prompt: &str, versions: &'a [T], default_version: T) -> Result<&'a T>
where
    T: Display + PartialEq,
{
    let default_idx = versions.iter()
        .position(|v| *v == default_version)
        .unwrap_or(0);

    let items = versions.iter().map(|v| {
        if *v == default_version {
            format!("{} (默认)", v)
        } else {
            v.to_string()
        }
    }).collect::<Vec<_>>();

    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default_idx)
        .items(&items)
        .interact()?;

    Ok(&versions[selected])
}

/// 配置工具函数：是/否选项
pub fn select_yes_no(prompt: &str, default_value: bool) -> Result<bool> {
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(if default_value { 0 } else { 1 })
        .items(&["是", "否"])
        .interact()?;
    
    Ok(selected == 0)
}

/// 配置工具函数：输入端口号
pub fn input_port(prompt: &str, default_port: u16) -> Result<u16> {
    Ok(Input::<u16>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default_port)
        .interact_text()?)
}

/// 配置工具函数：输入可选密码
pub fn input_optional_password(prompt: &str, default_value: Option<String>) -> Result<Option<String>> {
    let password = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .allow_empty(true)
        .default(default_value.unwrap_or_default())
        .interact_text()?;

    Ok(if password.is_empty() { None } else { Some(password) })
}
