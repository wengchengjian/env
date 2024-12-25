use std::error::Error;
use std::path::PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Input, Select};

pub mod environment;
pub mod env_config;
pub mod download;
pub mod check;
pub mod zip;
pub mod path;
pub mod installers;
pub mod args;
pub mod version;

use args::*;
use env_config::EnvConfig;
use environment::{environments::{configure_go, configure_java, configure_mongodb, configure_mysql, configure_node, configure_postgresql, configure_python, configure_redis, configure_rust}, select_version, switch_version};
use version::*;

/// 自定义Result类型，用于统一错误处理
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// 命令行参数结构体
#[derive(Parser)]
#[command(name = "env", version = "1.0.0", about="快速安装开发环境")]
#[command(long_about= "快速安装常见开发环境, 比如Java, Python3, Rust等等,也能作为环境检测工具使用")]
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
        flush: bool
    },

    Dev(DevEnvironmentArgs),

    /// 版本选择
    Choose {
        #[arg(value_enum)]
        name: ChooseEnvironment
    }
}


#[derive(Args,Clone, Debug)]
pub struct DevEnvironmentArgs {
    #[command(subcommand)]
    pub name: Option<DevEnvironment>,

    /// 安装所有支持的环境
    #[arg(short, long)]
    pub all: bool,
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

    /// 获取当前环境安装的所有版本
    pub fn get_versions(&self) -> Vec<String> {
        let name = self.get_name();
        let ret = vec![];
        // 从配置文件中读取所有安装版本
        ret
    }
}

/// 支持的开发环境枚举
#[derive(Subcommand, Clone, Debug)]
pub enum DevEnvironment {
    /// Java开发环境
    Java(JavaArgs),
    /// Python开发环境
    Python(PythonArgs),
    /// Node.js开发环境
    Node(NodeArgs),
    /// Rust开发环境
    Rust(RustArgs),
    /// Go开发环境
    Go(GoArgs),
    /// MySQL数据库
    MySQL(MySQLArgs),
    /// PostgreSQL数据库
    PostgreSQL(PostgreSQLArgs),
    /// MongoDB数据库
    MongoDB(MongoDBArgs),
    /// Redis数据库
    Redis(RedisArgs),
}

impl DevEnvironment {
    /// 获取环境名称
    pub fn get_name(&self) -> &'static str {
        match self {
            DevEnvironment::Java(_) => "Java",
            DevEnvironment::Python(_) => "Python",
            DevEnvironment::Node(_) => "Node.js",
            DevEnvironment::Rust(_) => "Rust",
            DevEnvironment::Go(_) => "Go",
            DevEnvironment::MySQL(_) => "MySQL",
            DevEnvironment::PostgreSQL(_) => "PostgreSQL",
            DevEnvironment::MongoDB(_) => "MongoDB",
            DevEnvironment::Redis(_) => "Redis",
        }
    }

    /// 获取环境描述
    pub fn get_description(&self) -> &'static str {
        match self {
            DevEnvironment::Java(_) => "Java开发环境",
            DevEnvironment::Python(_) => "Python开发环境",
            DevEnvironment::Node(_) => "Node.js开发环境",
            DevEnvironment::Rust(_) => "Rust开发环境",
            DevEnvironment::Go(_) => "Go开发环境",
            DevEnvironment::MySQL(_) => "MySQL数据库",
            DevEnvironment::PostgreSQL(_) => "PostgreSQL数据库",
            DevEnvironment::MongoDB(_) => "MongoDB数据库",
            DevEnvironment::Redis(_) => "Redis数据库",
        }
    }

    /// 获取版本号
    pub fn get_version(&self) -> Option<String> {
        match self {
            DevEnvironment::Java(args) => args.version.clone(),
            DevEnvironment::Python(args) => args.version.clone(),
            DevEnvironment::Node(args) => args.version.clone(),
            DevEnvironment::Rust(args) => args.version.clone(),
            DevEnvironment::Go(args) => args.version.clone(),
            DevEnvironment::MySQL(args) => args.version.clone(),
            DevEnvironment::PostgreSQL(args) => args.version.clone(),
            DevEnvironment::MongoDB(args) => args.version.clone(),
            DevEnvironment::Redis(args) => args.version.clone(),
        }
    }

    /// 获取所有支持的环境列表
    pub fn get_all() -> Vec<DevEnvironment> {
        vec![
            DevEnvironment::Java(JavaArgs::default()),
            DevEnvironment::Python(PythonArgs::default()),
            DevEnvironment::Node(NodeArgs::default()),
            DevEnvironment::Rust(RustArgs::default()),
            DevEnvironment::Go(GoArgs::default()),
            DevEnvironment::MySQL(MySQLArgs::default()),
            DevEnvironment::PostgreSQL(PostgreSQLArgs::default()),
            DevEnvironment::MongoDB(MongoDBArgs::default()),
            DevEnvironment::Redis(RedisArgs::default()),
        ]
    }
}

/// 交互式配置环境参数
pub fn configure_environment(env: &DevEnvironment) -> Result<DevEnvironment> {
    match env {
        DevEnvironment::Java(args) => configure_java(args),
        DevEnvironment::Python(args) => configure_python(args),
        DevEnvironment::Node(args) => configure_node(args),
        DevEnvironment::Rust(args) => configure_rust(args),
        DevEnvironment::Go(args) => configure_go(args),
        DevEnvironment::MySQL(args) => configure_mysql(args),
        DevEnvironment::PostgreSQL(args) => configure_postgresql(args),
        DevEnvironment::MongoDB(args) => configure_mongodb(args),
        DevEnvironment::Redis(args) => configure_redis(args)
    }
}

/// 安装所有支持的环境
pub async fn install_all(install_dir: &PathBuf) -> Result<()> {

    for env in DevEnvironment::get_all() {
        println!("正在安装 {}...", env.get_name());
        install_environment(&env, install_dir).await?;
    }
    Ok(())
}

/// 安装指定的环境
pub async fn install_environment(env: &DevEnvironment, install_dir: &PathBuf) -> Result<()> {
    match env {
        DevEnvironment::Java(args) => installers::java::install_java(install_dir, &args.version).await?,
        // TODO: 添加其他环境的安装
        _ => println!("暂不支持安装 {}", env.get_name()),
    }

    let mut config = EnvConfig::load_deserialize();
    config.add_args(env);
    config.save();

    Ok(())
    
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


pub fn choose_version(env: &ChooseEnvironment) -> Result<()> {
    let config = EnvConfig::load_deserialize();
    
    let versions = config.get_install_versions(env);
    let current_version = config.get_current_version(env);
    
    if current_version.is_none() || versions.is_empty() {
        println!("未找到 {} 的当前版本", env.get_name());
        return Ok(());
    }
    let selected_version = select_version("选择版本", &versions, current_version.unwrap())?;
    
    // 切换版本
    switch_version(env, selected_version);

    let mut config = EnvConfig::load_deserialize();
    config.set_current_version(env, selected_version);
    config.save();

    Ok(())
}

pub async fn choose_and_install_from(env: &DevEnvironment, install_dir: &PathBuf) -> Result<()> {
    let configured_env = configure_environment(env)?;

    install_environment(&configured_env, install_dir).await?;

    Ok(())
}

/// 显示交互式选择菜单并安装选中的环境
pub async fn choose_and_install(install_dir: &PathBuf) -> Result<()> {
    let environments = DevEnvironment::get_all();
    let items: Vec<String> = environments
        .iter()
        .map(|e| format!("{} - {}", e.get_name(), e.get_description()))
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
    let mut configured_environments = Vec::new();
    for &index in selections.iter() {
        let env = &environments[index];
        let configured_env = configure_environment(env)?;
        configured_environments.push(configured_env);
    }

    // 安装配置后的环境
    for env in configured_environments {
        println!("正在安装 {}...", env.get_name());
        install_environment(&env, install_dir).await?;
    }

    Ok(())
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
    PathBuf::from( get_home_dir()).join(".dev_env")
}

/// 获取系统临时目录路径
pub fn get_temp_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(std::env::var("TEMP").unwrap_or_else(|_| ".".to_string()))
    } else {
        PathBuf::from("/tmp")
    }
}