use std::path::PathBuf;
use std::str::FromStr;

use colored::Colorize;

use crate::env_config::{ConfigTryLoad, EnvConfig, JavaConfig};
use crate::path::DEFAULT_JDK_VERSION;
use crate::{Result, DevEnvironment};
use crate::args::*;
use crate::version::*;
use super::{select_version, select_yes_no, input_port, input_optional_password};


pub fn flush_env_config(env_config: &mut EnvConfig) {
    println!("\n刷新配置:");
    
    let home_dir = PathBuf::from(&env_config.install_path);
    if !home_dir.exists() {
        println!("{}\n", "安装目录不存在".red());
        return;
    }
    env_config.java = JavaConfig::try_load(&home_dir);
    
}

pub fn configure_java(args: &JavaArgs) -> Result<DevEnvironment> {
    if args.version.is_some() {
        return Ok(DevEnvironment::Java(JavaArgs {
            version: args.version.clone(),
        }));
    }
    println!("\n配置Java环境参数:");
    let versions = JavaVersion::all();
    let config = EnvConfig::load_deserialize();

    let default_version = {
        if let Some(java) = &config.java {
            JavaVersion::from_str(&java.version.current_version.clone().unwrap_or(DEFAULT_JDK_VERSION.to_string())).unwrap()
        } else {
            JavaVersion::JDK17
        }
    };
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    Ok(DevEnvironment::Java(JavaArgs {
        version: Some(version),
    }))
}

pub fn configure_python(args: &PythonArgs) -> Result<DevEnvironment> {
    println!("\n配置Python环境参数:");
    let versions = PythonVersion::all();
    let default_version = PythonVersion::Python311;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let pip = select_yes_no("是否安装pip", args.pip)?;

    Ok(DevEnvironment::Python(PythonArgs {
        version: Some(version),
        pip,
    }))
}

pub fn configure_node(args: &NodeArgs) -> Result<DevEnvironment> {
    println!("\n配置Node.js环境参数:");
    let versions = NodeVersion::all();
    let default_version = NodeVersion::Node20;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let npm = select_yes_no("是否安装npm", args.npm)?;

    Ok(DevEnvironment::Node(NodeArgs {
        version: Some(version),
        npm,
    }))
}

pub fn configure_rust(args: &RustArgs) -> Result<DevEnvironment> {
    println!("\n配置Rust环境参数:");
    let versions = RustVersion::all();
    let default_version = RustVersion::Rust174;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let cargo = select_yes_no("是否安装cargo", args.cargo)?;

    Ok(DevEnvironment::Rust(RustArgs {
        version: Some(version),
        cargo,
    }))
}

pub fn configure_go(args: &GoArgs) -> Result<DevEnvironment> {
    println!("\n配置Go环境参数:");
    let versions = GoVersion::all();
    let default_version = GoVersion::Go121;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let set_gopath = select_yes_no("是否设置GOPATH", args.set_gopath)?;

    Ok(DevEnvironment::Go(GoArgs {
        version: Some(version),
        set_gopath,
    }))
}

pub fn configure_mysql(args: &MySQLArgs) -> Result<DevEnvironment> {
    println!("\n配置MySQL环境参数:");
    let versions = MySQLVersion::all();
    let default_version = MySQLVersion::MySQL80;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let port = input_port("端口号", args.port)?;
    let root_password = input_optional_password("root密码 (可选)", args.root_password.clone())?;

    Ok(DevEnvironment::MySQL(MySQLArgs {
        version: Some(version),
        port,
        root_password,
    }))
}

pub fn configure_postgresql(args: &PostgreSQLArgs) -> Result<DevEnvironment> {
    println!("\n配置PostgreSQL环境参数:");
    let versions = PostgreSQLVersion::all();
    let default_version = PostgreSQLVersion::PostgreSQL16;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let port = input_port("端口号", args.port)?;

    Ok(DevEnvironment::PostgreSQL(PostgreSQLArgs {
        version: Some(version),
        port,
    }))
}

pub fn configure_mongodb(args: &MongoDBArgs) -> Result<DevEnvironment> {
    println!("\n配置MongoDB环境参数:");
    let versions = MongoDBVersion::all();
    let default_version = MongoDBVersion::MongoDB70;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let port = input_port("端口号", args.port)?;

    Ok(DevEnvironment::MongoDB(MongoDBArgs {
        version: Some(version),
        port,
    }))
}

pub fn configure_redis(args: &RedisArgs) -> Result<DevEnvironment> {
    println!("\n配置Redis环境参数:");
    let versions = RedisVersion::all();
    let default_version = RedisVersion::Redis72;
    let selected_version = select_version("选择版本", &versions, default_version)?;
    let version = selected_version.get_version().to_string();

    let port = input_port("端口号", args.port)?;
    let password = input_optional_password("密码 (可选)", args.password.clone())?;

    Ok(DevEnvironment::Redis(RedisArgs {
        version: Some(version),
        port,
        password,
    }))
}
