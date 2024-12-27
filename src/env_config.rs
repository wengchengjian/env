use crate::deduplicate;
use crate::get_env_home_dir;
use anyhow::Ok;
use anyhow::Result;
use config::Config;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

const DEFAULT_ENV_CONFIG: &'static str = include_str!("../.env.config.default.json");

lazy_static! {
    pub static ref ENV_CONFIG: EnvConfig = EnvConfig::load_deserialize().expect("加载环境配置失败");
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvConfig {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub install_path: String,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub arch_mapping: HashMap<String, HashMap<String, String>>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub environments: Vec<Environment>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed: Option<Vec<InstalledEnvironment>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstalledEnvironment {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_dir: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub installed_versions: Vec<String>,
}

impl InstalledEnvironment {
    pub fn new(name: &str, version: &str, home_dir: &str) -> InstalledEnvironment {
        InstalledEnvironment {
            name: name.to_string(),
            current_version: Some(version.to_string()),
            home_dir: Some(home_dir.to_string()),
            installed_versions: vec![version.to_string()],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<EnvironmentInteractArgs>,
    pub format: Value,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub executable: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,
    pub repository: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvironmentInteractArgs {
    pub name: String,
    pub description: String,

    #[serde(rename = "type")]
    pub type_: String,
    pub default: String,
    pub options: Vec<String>,
    pub select_description: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvironmentSelectArgs {
    pub name: String,

    pub value: String,
}

impl EnvConfig {
    pub fn get_enviroment(&self, name: &str) -> Option<Environment> {
        self.environments
            .iter()
            .find(|env| env.name.to_lowercase() == name.to_lowercase())
            .cloned()
    }

    pub fn switch_version(name: &str, version: &str, install_dir: &PathBuf) -> Result<()> {
        let install_dir = install_dir.to_str().unwrap();

        let mut config = ENV_CONFIG.clone();
        let new_installed = InstalledEnvironment::new(name, version, install_dir);

        if let Some(installed) = &mut config.installed {
            if let Some(env) = installed
                .iter_mut()
                .find(|env| env.name.to_lowercase() == name.to_lowercase())
            {
                env.current_version = Some(version.to_string());
                env.home_dir = Some(install_dir.to_string());
                // 去重
                env.installed_versions.push(version.to_string());
                env.installed_versions = deduplicate(&env.installed_versions);
            } else {
                installed.push(new_installed);
            }
        } else {
            config.installed = Some(vec![new_installed]);
        }

        EnvConfig::save(&config)?;

        Ok(())
    }

    pub fn get_current_version(&self, name: &str) -> Option<String> {
        if let Some(installed) = &self.installed {
            let env = installed
                .iter()
                .find(|env| env.name.to_lowercase() == name.to_lowercase())
                .unwrap();
            return env.current_version.clone();
        }
        None
    }

    pub fn get_install_versions(&self, name: &str) -> Vec<String> {
        if let Some(installed) = &self.installed {
            let env = installed
                .iter()
                .find(|env| env.name.to_lowercase() == name.to_lowercase())
                .unwrap();
            return env.installed_versions.clone();
        }
        vec![]
    }

    pub fn save(config: &EnvConfig) -> Result<()> {
        let config = serde_json::to_string_pretty(config)?;

        fs::write(get_home_config_path(), config)?;
        Ok(())
    }

    pub fn init() -> Result<()> {
        let home_config_path = get_home_config_path();

        if home_config_path.exists() {
            return Ok(());
        }

        let default_env_config_str = DEFAULT_ENV_CONFIG;

        let mut default_env_config: EnvConfig = serde_json::from_str(default_env_config_str)?;

        // 设置默认安装目录
        if default_env_config.install_path.is_empty() {
            default_env_config.install_path = get_env_home_dir().to_str().unwrap().to_string();
        }

        // 写入配置
        serde_json::to_writer_pretty(File::create(home_config_path)?, &default_env_config)?;
        Ok(())
    }

    pub fn load() -> Result<Config> {
        let home_config = get_home_config_path();

        // 初始化home配置文件
        EnvConfig::init()?;

        let home_config = config::File::with_name(home_config.as_os_str().to_str().unwrap());

        let local_config = ".env.config.json";

        let local_exist = config_exist(local_config);

        let local_config = config::File::with_name(&local_config);

        let mut setting = Config::builder().add_source(home_config);

        if local_exist {
            setting = setting.add_source(local_config);
        }
        let setting = setting.build()?;
        Ok(setting)
    }
    /// 加载配置
    pub fn load_deserialize() -> Result<EnvConfig> {
        let setting = EnvConfig::load()?;

        let setting = setting.try_deserialize::<EnvConfig>()?;
        Ok(setting)
    }
}

pub fn config_exist(filename: &str) -> bool {
    let path = Path::new(filename);
    path.exists()
}

pub fn get_home_config_path() -> PathBuf {
    PathBuf::from(get_env_home_dir()).join(".env.config.json")
}

pub fn flush_env_config() -> anyhow::Result<()> {
    // 初始化home配置文件
    let mut config: EnvConfig = serde_json::from_str(DEFAULT_ENV_CONFIG)?;

    find_all_installed_version(&mut config)?;

    EnvConfig::save(&config)?;

    Ok(())
}

pub fn find_all_installed_version(env_config: &mut EnvConfig) -> anyhow::Result<()> {
    let install_dir = PathBuf::from(&env_config.install_path);

    if !install_dir.exists() {
        return Ok(());
    }
    let mut installeds = vec![];
    //遍历该目录下的一级目录，获取所有的版本
    let dirs = fs::read_dir(install_dir).unwrap();
    for dir in dirs {
        let dir = dir.unwrap().path();

        let name = dir.file_name().unwrap().to_str().unwrap();
        // 检查这个环境在不在配置中
        if !env_config
            .environments
            .iter()
            .any(|env| env.name.to_lowercase() == name.to_lowercase())
        {
            continue;
        }
        let installed_version: InstalledEnvironment = find_version_from_dir(&dir)?;

        installeds.push(installed_version);
    }

    env_config.installed = Some(installeds);

    Ok(())
}

pub fn find_version_from_dir(dir: &PathBuf) -> Result<InstalledEnvironment> {
    let name = dir.file_name().unwrap().to_str().unwrap().to_lowercase();

    let dirs = fs::read_dir(dir)?;

    let mut versions = vec![];

    for dir in dirs {
        let dir = dir?.path();
        let filename = dir.file_name().unwrap().to_str().unwrap().to_lowercase();

        if filename.contains(&name) {
            let version = filename.split("-").nth(1).unwrap();
            versions.push(version.to_string());
        }
    }
    Ok(InstalledEnvironment {
        name: name.to_string(),
        current_version: None,
        home_dir: None,
        installed_versions: versions,
    })
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
