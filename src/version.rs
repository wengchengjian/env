use std::{fmt, str::FromStr};

/// Java版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JavaVersion {
    JDK8,
    JDK11,
    JDK17,
    JDK21,
}

impl FromStr for JavaVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<JavaVersion, ()> {
        match s {
            "8.0.392" => Ok(JavaVersion::JDK8),
            "11.0.21" => Ok(JavaVersion::JDK11),
            "17.0.9" => Ok(JavaVersion::JDK17),
            "21.0.1" => Ok(JavaVersion::JDK21),
            _ => Err(()),
        }
    }
}

impl JavaVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            JavaVersion::JDK8 => "8.0.392",
            JavaVersion::JDK11 => "11.0.21",
            JavaVersion::JDK17 => "17.0.9",
            JavaVersion::JDK21 => "21.0.1",
        }
    }

    pub fn all() -> &'static [JavaVersion] {
        &[
            JavaVersion::JDK8,
            JavaVersion::JDK11,
            JavaVersion::JDK17,
            JavaVersion::JDK21,
        ]
    }
}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JDK {} ({})", self.get_version(), match self {
            JavaVersion::JDK8 => "LTS",
            JavaVersion::JDK11 => "LTS",
            JavaVersion::JDK17 => "LTS",
            JavaVersion::JDK21 => "LTS",
        })
    }
}

/// Python版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PythonVersion {
    Python38,
    Python39,
    Python310,
    Python311,
    Python312,
}

impl PythonVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            PythonVersion::Python38 => "3.8.18",
            PythonVersion::Python39 => "3.9.18",
            PythonVersion::Python310 => "3.10.13",
            PythonVersion::Python311 => "3.11.7",
            PythonVersion::Python312 => "3.12.1",
        }
    }

    pub fn all() -> &'static [PythonVersion] {
        &[
            PythonVersion::Python38,
            PythonVersion::Python39,
            PythonVersion::Python310,
            PythonVersion::Python311,
            PythonVersion::Python312,
        ]
    }
}

impl fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Python {}", self.get_version())
    }
}

/// Node.js版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeVersion {
    Node16,
    Node18,
    Node20,
    Node21,
}

impl NodeVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            NodeVersion::Node16 => "16.20.2",
            NodeVersion::Node18 => "18.19.0",
            NodeVersion::Node20 => "20.10.0",
            NodeVersion::Node21 => "21.5.0",
        }
    }

    pub fn all() -> &'static [NodeVersion] {
        &[
            NodeVersion::Node16,
            NodeVersion::Node18,
            NodeVersion::Node20,
            NodeVersion::Node21,
        ]
    }
}

impl fmt::Display for NodeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node.js {} ({})", self.get_version(), match self {
            NodeVersion::Node16 => "维护版本",
            NodeVersion::Node18 => "LTS",
            NodeVersion::Node20 => "LTS",
            NodeVersion::Node21 => "当前版本",
        })
    }
}

/// Rust版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RustVersion {
    Rust173,
    Rust174,
    Rust175,
}

impl RustVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            RustVersion::Rust173 => "1.73.0",
            RustVersion::Rust174 => "1.74.1",
            RustVersion::Rust175 => "1.75.0",
        }
    }

    pub fn all() -> &'static [RustVersion] {
        &[
            RustVersion::Rust173,
            RustVersion::Rust174,
            RustVersion::Rust175,
        ]
    }
}

impl fmt::Display for RustVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rust {}", self.get_version())
    }
}

/// Go版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GoVersion {
    Go120,
    Go121,
    Go122Beta,
}

impl GoVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            GoVersion::Go120 => "1.20.12",
            GoVersion::Go121 => "1.21.5",
            GoVersion::Go122Beta => "1.22.0-beta1",
        }
    }

    pub fn all() -> &'static [GoVersion] {
        &[
            GoVersion::Go120,
            GoVersion::Go121,
            GoVersion::Go122Beta,
        ]
    }
}

impl fmt::Display for GoVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Go {}", self.get_version())
    }
}

/// MySQL版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MySQLVersion {
    MySQL57,
    MySQL80,
}

impl MySQLVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            MySQLVersion::MySQL57 => "5.7.44",
            MySQLVersion::MySQL80 => "8.0.35",
        }
    }

    pub fn all() -> &'static [MySQLVersion] {
        &[
            MySQLVersion::MySQL57,
            MySQLVersion::MySQL80,
        ]
    }
}

impl fmt::Display for MySQLVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MySQL {}", self.get_version())
    }
}

/// PostgreSQL版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PostgreSQLVersion {
    PostgreSQL13,
    PostgreSQL14,
    PostgreSQL15,
    PostgreSQL16,
}

impl PostgreSQLVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            PostgreSQLVersion::PostgreSQL13 => "13.13",
            PostgreSQLVersion::PostgreSQL14 => "14.10",
            PostgreSQLVersion::PostgreSQL15 => "15.5",
            PostgreSQLVersion::PostgreSQL16 => "16.1",
        }
    }

    pub fn all() -> &'static [PostgreSQLVersion] {
        &[
            PostgreSQLVersion::PostgreSQL13,
            PostgreSQLVersion::PostgreSQL14,
            PostgreSQLVersion::PostgreSQL15,
            PostgreSQLVersion::PostgreSQL16,
        ]
    }
}

impl fmt::Display for PostgreSQLVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PostgreSQL {}", self.get_version())
    }
}

/// MongoDB版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MongoDBVersion {
    MongoDB60,
    MongoDB70,
}

impl MongoDBVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            MongoDBVersion::MongoDB60 => "6.0.12",
            MongoDBVersion::MongoDB70 => "7.0.4",
        }
    }

    pub fn all() -> &'static [MongoDBVersion] {
        &[
            MongoDBVersion::MongoDB60,
            MongoDBVersion::MongoDB70,
        ]
    }
}

impl fmt::Display for MongoDBVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MongoDB {}", self.get_version())
    }
}

/// Redis版本枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RedisVersion {
    Redis62,
    Redis70,
    Redis72,
}

impl RedisVersion {
    pub fn get_version(&self) -> &'static str {
        match self {
            RedisVersion::Redis62 => "6.2.14",
            RedisVersion::Redis70 => "7.0.14",
            RedisVersion::Redis72 => "7.2.3",
        }
    }

    pub fn all() -> &'static [RedisVersion] {
        &[
            RedisVersion::Redis62,
            RedisVersion::Redis70,
            RedisVersion::Redis72,
        ]
    }
}

impl fmt::Display for RedisVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Redis {}", self.get_version())
    }
}
