use std::{fs::{self, File}, path::{Path, PathBuf}};

use config::Config;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{check::get_java_version, deduplicate, get_env_home_dir, ChooseEnvironment, DevEnvironment};



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvConfig {
    pub install_path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub java: Option<JavaConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<PythonConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust: Option<RustConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub go: Option<GoConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql: Option<MySQLConfig>,
}

impl EnvConfig {

    pub fn new() -> EnvConfig {
        EnvConfig {
            install_path: get_env_home_dir().as_os_str().to_str().unwrap().to_string(),
            java: None,
            python: None,
            node: None,
            rust: None,
            go: None,
            mysql: None,
        }
    }

    pub fn set_current_version(&mut self, env: &ChooseEnvironment, version: &String) {
        match env {
            ChooseEnvironment::Java => {
                if let Some(java) = &mut self.java {
                    java.version.current_version = Some(version.clone());
                } else {
                    self.java = Some(JavaConfig::new(Some(version.clone()), vec![version.clone()]));
                }
            },
            _ => {}
        }
    }

    pub fn add_args(&mut self, env: &DevEnvironment) {
        match env {
            DevEnvironment::Java(args) => {    
                let version = args.version.clone().unwrap();

                if let Some(java) = &mut self.java {
                    java.version.install_versions.push(version.clone());
                    //去重
                    java.version.install_versions = deduplicate(&java.version.install_versions);
                    java.version.current_version = Some(version)
                } else {
                    self.java = Some(JavaConfig::new(Some(version.clone()), vec![version]));
                }
                
            }
            _=> {}
        }
    }

    pub fn save(&self) {
        let config = serde_json::to_string_pretty(self).unwrap();
        fs::write(get_home_config_path(), config).unwrap();
    }

    pub fn get_current_version(&self, name: &ChooseEnvironment) -> Option<String> {
        match name {
            ChooseEnvironment::Java => {
                if let Some(java) = &self.java {
                    return java.version.current_version.clone()
                }
            },
            ChooseEnvironment::Python => {
                if let Some(python) = &self.python {
                    return python.version.current_version.clone()
                }
            },
            ChooseEnvironment::Node => {
                if let Some(node) = &self.node {
                    return node.version.current_version.clone()
                }
            },
            ChooseEnvironment::Rust => {
                if let Some(rust) = &self.rust {
                    return rust.version.current_version.clone()
                }
            },
            _ => {}
        }
        None
    }

    pub fn get_install_versions(&self, name: &ChooseEnvironment) -> Vec<String> {
        match name {
            ChooseEnvironment::Java => {
                if let Some(java) = &self.java {
                    return java.version.install_versions.clone()
                }
            },
            ChooseEnvironment::Python => {
                if let Some(python) = &self.python {
                    return python.version.install_versions.clone()
                }
            },
            ChooseEnvironment::Node => {
                if let Some(node) = &self.node {
                    return node.version.install_versions.clone()
                }
            },
            ChooseEnvironment::Rust => {
                if let Some(rust) = &self.rust {
                    return rust.version.install_versions.clone()
                }
            },
            ChooseEnvironment::Go => {
                if let Some(go) = &self.go {
                    return go.version.install_versions.clone()
                }
            },
            ChooseEnvironment::MySQL => {
                if let Some(mysql) = &self.mysql {
                    return mysql.version.install_versions.clone()
                }
            },

            _ => {}
        }

        vec![]
    }


    pub fn init() {
        let home_config_path = get_home_config_path();

        if home_config_path.exists() {
            return;
        }
        let default_env_config = json!({
            "install_path": get_env_home_dir().as_os_str().to_str().unwrap().to_string(),
        });

        // 写入配置
        serde_json::to_writer_pretty(
            File::create(home_config_path).unwrap(),
            &default_env_config,
        ).unwrap();
    }

    pub fn load() -> Config {
        let home_config = get_home_config_path();

        // 初始化home配置文件
        EnvConfig::init();

        let home_config = config::File::with_name(home_config.as_os_str().to_str().unwrap());

        let local_config = ".env.config.json";

        let local_exist = config_exist(local_config);

        let local_config = config::File::with_name(&local_config);

        let mut setting = Config::builder().add_source(home_config);

        if local_exist {
            setting = setting.add_source(local_config);
        }
        let setting = setting.build().expect("无法读取配置文件");
        setting
    }
    /// 加载配置 
    pub fn load_deserialize() -> EnvConfig {
        let setting = EnvConfig::load();

        let setting = setting.try_deserialize::<EnvConfig>().expect("配置文件格式错误，请检查");
        setting
    }

}

pub fn config_exist(filename: &str) -> bool {
    let path = Path::new(filename);
    path.exists()
}


pub fn get_home_config_path() -> PathBuf {
    PathBuf::from(get_env_home_dir()).join(".env.config.json")
}

pub trait ConfigTryLoad {
    fn try_load(dir: &PathBuf) -> Option<Self> where Self: Sized;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JavaConfig {
    pub version: EnvVersion,
}

impl JavaConfig {
    pub fn new(current_version: Option<String>, install_versions: Vec<String>) -> JavaConfig {
        JavaConfig {
            version: EnvVersion {
                current_version,
                install_versions,
            }
        }
    }
}

impl ConfigTryLoad for JavaConfig {
    fn try_load(dir: &PathBuf) -> Option<Self> where Self: Sized {
        let path = dir.join("java");
        if !path.exists() {
            return None;
        }
        // 通过命令行获取Java版本
        let current_version = get_java_version();
        //遍历该目录下的一级目录，获取所有的版本
        let dirs = fs::read_dir(path).unwrap();
        let mut install_versions = vec![];
        for dir in dirs {
            let dir = dir.unwrap();
            let path = dir.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_str().unwrap();
                let sp = name.split("-").collect::<Vec<&str>>();
                if sp.len() < 2 {
                    continue;
                }
                let version = sp[1].to_string();
                install_versions.push(version);
            }
        }

        Some(JavaConfig::new(current_version, install_versions))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedisConfig {
    pub version: EnvVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub port: u16,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PythonConfig {
    pub version: EnvVersion,

}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeConfig {
    pub version: EnvVersion,

}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustConfig {
    pub version: EnvVersion,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoConfig {
    pub version: EnvVersion,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MySQLConfig {
    pub version: EnvVersion,
    pub root: String,
    pub root_password: String,
    pub port: u16
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvVersion {
    pub current_version: Option<String>,
    pub install_versions: Vec<String>
}

impl EnvVersion {
    fn get_current_version(&self) -> &Option<String> {
        &self.current_version
    }

    fn get_install_versions(&self) -> &Vec<String> {
        &self.install_versions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn test_init_config() {
        let env_config = EnvConfig::load();
        println!("{:?}", env_config);
    }
}