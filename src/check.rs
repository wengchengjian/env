use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use crate::env_config::EnvConfig;

pub fn validate_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    for part in parts {
        if part.is_empty() {
            return false;
        }
        for c in part.chars() {
            if !c.is_digit(10) {
                return false;
            }
        }
    }
    true
}

/// 命令行获取Java版本
pub fn get_java_version_from(output: &str) -> Option<String> {
    let version = output.split(" ").nth(1).unwrap_or("unknown");

    if version == "unknown" || !validate_version(version) {
        return None;
    }

    Some(version.to_string())
}

/// 命令行获取Java版本
pub fn get_java_version() -> Option<String> {
    let output = Command::new("java").arg("--version").output();

    let output = match output {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(e) => {
            eprintln!("获取Java版本失败: {}", e);
            return None;
        }
    };

    let version = output.split(" ").nth(1).unwrap_or("unknown");

    if version == "unknown" || !validate_version(version) {
        return None;
    }

    Some(version.to_string())
}

pub fn is_downloaded(name: &str, version: &str) -> bool {
    let install_dir = PathBuf::from(&EnvConfig::load_deserialize().unwrap().install_path);
    let install_dir = install_dir.join(name);
    let download_dir = install_dir.join(format!("{}-{}", name, version));
    download_dir.exists()
}

/// 检查Java环境
pub fn check_java_environment(version: &str) -> bool {
    let java_check = Command::new("java").arg("--version").output();

    match java_check {
        Ok(output) => {
            if output.status.success() {
                // 可以打印 Java 版本信息
                if let Some(stdout) = String::from_utf8(output.stdout).ok() {
                    let current_version = get_java_version_from(&stdout);
                    if let Some(current_version) = current_version {
                        if current_version == version {
                            println!("Java 环境已安装");
                            println!("Java 版本信息: {}", stdout);
                            return true;
                        }
                    }
                }
            } else {
                println!("Java 环境未正确安装或无法正常工作");
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("未安装Java")
            }
            ErrorKind::PermissionDenied => {
                eprintln!("权限不足")
            }
            _ => {
                eprintln!("检查Java环境出错:{}", e)
            }
        },
    };

    false
}

#[cfg(test)]
mod tests {
    use crate::check::{check_java_environment, get_java_version};

    #[test]
    fn test_get_java_version() {
        let version = get_java_version();
        assert_eq!(version, Some("17.0.12".to_string()));
    }

    #[test]
    fn test_check_java_environment() {
        let version = "17.0.12";
        assert!(check_java_environment(&version));
    }
}
