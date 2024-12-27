use anyhow::{anyhow, Ok, Result};
use check::is_downloaded;
use clap::{Args, Parser, Subcommand, ValueEnum};
use dialoguer::Password;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use env_config::{EnvConfig, Environment, EnvironmentSelectArgs, ENV_CONFIG};
use environment::switch_version;
use serde_json::{json, Value};
use std::ops::Index;
use std::path::PathBuf;
use std::{fs, usize};
use zip::auto_unzip;

pub mod check;
pub mod download;
pub mod env_config;
pub mod environment;
pub mod install;
pub mod path;
pub mod zip;

/// 自定义Result类型，用于统一错误处理

/// 命令行参数结构体
#[derive(Parser)]
#[command(name = "env", version = "1.0.0", about = "快速安装开发环境")]
#[command(
    long_about = "快速安装常见开发环境, 比如Java, Python3, Rust等等,也能作为环境检测工具使用"
)]
pub struct EnvArgs {
    /// 子命令指定要安装的环境
    #[command(subcommand)]
    pub command: Option<EnvSubCommand>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum EnvSubCommand {
    /// 全局配置
    Config {
        /// 安装目录
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// 刷新配置
        #[arg(long)]
        flush: bool,
    },

    Dev(DevEnvironmentArgs),

    /// 版本选择
    Choose {
        #[arg(value_enum)]
        name: ChooseEnvironment,
    },
}

#[derive(Args, Clone, Debug)]
pub struct DevEnvironmentArgs {
    // 安装所有支持的环境
    // #[arg(short, long)]
    // pub all: bool,
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, ValueEnum)]
pub enum ChooseEnvironment {
    Java,
    Python,
    Node,
    Rust,
    Go,
    MySQL,
    PostgreSQL,
    MongoDB,
    Redis,
}

impl ChooseEnvironment {
    pub fn get_name(&self) -> &'static str {
        match self {
            ChooseEnvironment::Java => "Java",
            ChooseEnvironment::Python => "Python",
            ChooseEnvironment::Node => "Node.js",
            ChooseEnvironment::Rust => "Rust",
            ChooseEnvironment::Go => "Go",
            ChooseEnvironment::MySQL => "MySQL",
            ChooseEnvironment::PostgreSQL => "PostgreSQL",
            ChooseEnvironment::MongoDB => "MongoDB",
            ChooseEnvironment::Redis => "Redis",
        }
    }
}

pub fn deduplicate<T: Eq + std::hash::Hash + Clone>(arr: &[T]) -> Vec<T> {
    let mut set = std::collections::HashSet::new();
    let mut result = Vec::new();
    for element in arr {
        if set.insert(element) {
            result.push(element.clone());
        }
    }
    result
}

/// 获取用户主目录路径
pub fn get_home_dir() -> String {
    if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    }
}

/// 获取env程序主目录路径
pub fn get_env_home_dir() -> PathBuf {
    PathBuf::from(get_home_dir()).join(".dev_env")
}

/// 获取系统临时目录路径
pub fn get_temp_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(std::env::var("TEMP").unwrap_or_else(|_| ".".to_string()))
    } else {
        PathBuf::from("/tmp")
    }
}
