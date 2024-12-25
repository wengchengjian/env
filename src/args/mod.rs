use clap::Args;

/// Java安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct JavaArgs {
    /// 版本号，例如: 17.0.2
    #[arg(short, long)]
    pub version: Option<String>,
}

/// Python安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct PythonArgs {
    /// 版本号，例如: 3.9.0
    #[arg(short, long, default_value = "3.11.7")]
    pub version: Option<String>,
    /// 是否安装pip
    #[arg(long, default_value = "true")]
    pub pip: bool,
}

/// Node.js安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct NodeArgs {
    /// 版本号，例如: 16.13.0
    #[arg(short, long, default_value = "20.10.0")]
    pub version: Option<String>,
    /// 是否安装npm
    #[arg(long, default_value = "true")]
    pub npm: bool,
}

/// Rust安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct RustArgs {
    /// 版本号，例如: 1.70.0
    #[arg(short, long, default_value = "1.74.1")]
    pub version: Option<String>,
    /// 是否安装cargo
    #[arg(long, default_value = "true")]
    pub cargo: bool,
}

/// Go安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct GoArgs {
    /// 版本号，例如: 1.19.0
    #[arg(short, long, default_value = "1.21.5")]
    pub version: Option<String>,
    /// 是否设置GOPATH
    #[arg(long, default_value = "true")]
    pub set_gopath: bool,
}

/// MySQL安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct MySQLArgs {
    /// 版本号，例如: 8.0.0
    #[arg(short, long, default_value = "8.0.35")]
    pub version: Option<String>,
    /// root密码
    #[arg(long)]
    pub root_password: Option<String>,
    /// 端口号
    #[arg(long, default_value = "3306")]
    pub port: u16,
}

/// PostgreSQL安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct PostgreSQLArgs {
    /// 版本号，例如: 14.0
    #[arg(short, long, default_value = "16.1")]
    pub version: Option<String>,
    /// 端口号
    #[arg(long, default_value = "5432")]
    pub port: u16,
}

/// MongoDB安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct MongoDBArgs {
    /// 版本号，例如: 5.0.0
    #[arg(short, long, default_value = "7.0.4")]
    pub version: Option<String>,
    /// 端口号
    #[arg(long, default_value = "27017")]
    pub port: u16,
}

/// Redis安装参数
#[derive(Args, Clone, Debug, Default)]
pub struct RedisArgs {
    /// 版本号，例如: 6.2.0
    #[arg(short, long, default_value = "7.2.3")]
    pub version: Option<String>,
    /// 端口号
    #[arg(long, default_value = "6379")]
    pub port: u16,
    /// 是否设置密码
    #[arg(long)]
    pub password: Option<String>,
}
