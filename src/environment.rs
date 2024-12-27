use crate::{
    env_config::{EnvConfig, Environment, ENV_CONFIG},
    path::{set_persistent_env, set_persistent_path},
    Result,
};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Password, Select};
use serde_json::Value;
use std::{collections::HashMap, env, path::PathBuf};

pub fn get_install_dir(env: &Environment, version: &str) -> PathBuf {
    let name = env.name.as_str();
    let install_dir = PathBuf::from(&ENV_CONFIG.install_path);
    let install_dir = install_dir.join(name);
    install_dir.join(format!("{}-{}", name, version))
}

pub fn get_vars(env: &Environment, version: &str) -> HashMap<String, String> {
    let install_dir = get_install_dir(env, version);
    // HOME目录
    let home_dir = install_dir.to_str().unwrap();
    let mut vars = HashMap::new();
    vars.insert("INSTALL_DIR".to_string(), home_dir.to_string());

    for (key, val) in env::vars() {
        vars.insert(key, val);
    }
    vars
}

pub fn handle_vars(val: &str, vars: &HashMap<String, String>) -> String {
    let mut value = val.to_string();
    for (var, val) in vars {
        value = value.replace(&format!("%{}%", var), val);
    }
    value
}

pub fn switch_version(env: &Environment, version: &str) -> Result<()> {
    let name = env.name.as_str();

    let vars = get_vars(env, version);

    let environments = &env.environment;
    let exec = &env.executable;

    // 设置环境变量
    println!("正在设置环境变量...");
    for (key, value) in environments {
        // 处理环境变量
        let value = handle_vars(value, &vars);
        set_persistent_env(&key, &value)?;
    }

    println!("添加可执行Path...");
    let mut path = PathBuf::new();
    for val in exec {
        // 处理环境变量
        let value = handle_vars(val, &vars);
        path = path.join(value);
    }

    let path = path.to_str().unwrap();
    set_persistent_path(None, path)?;

    println!("{}", "切换版本完成!".green());
    println!("版本: {}", version);

    let install_dir = get_install_dir(env, version);

    // 更新配置
    EnvConfig::switch_version(name, version, &install_dir)?;

    Ok(())
}

pub fn configure_environment(env: &Environment) -> Value {
    let args = &env.args;

    let mut ret = HashMap::new();
    for arg in args {
        let arg_type = arg.type_.as_str();
        let description = &arg.description;
        let options = &arg.options;
        let default_idx = options.iter().position(|v| v == &arg.default).unwrap_or(0);
        let select_description = &arg.select_description;
        let mut i = 0;

        let items = options
            .iter()
            .map(|v| {
                let sd = {
                    if i < select_description.len() {
                        select_description[i].clone()
                    } else {
                        String::new()
                    }
                };
                i += 1;
                if sd.is_empty() {
                    v.clone()
                } else {
                    format!("{} - {}", v, sd)
                }
            })
            .collect::<Vec<String>>();

        let mut value = Value::Null;

        match arg_type {
            "input" => {
                value = Value::String(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt(description)
                        .default(arg.default.clone())
                        .interact_text()
                        .unwrap(),
                );
            }
            "select" => {
                let selected = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(description)
                    .default(default_idx)
                    .items(&items)
                    .interact()
                    .unwrap();
                value = Value::String(options[selected].clone());
            }
            "multi-select" => {
                let selected = MultiSelect::with_theme(&ColorfulTheme::default())
                    .with_prompt(description)
                    .defaults(&vec![false; items.len()])
                    .items(&items)
                    .interact()
                    .unwrap();
                let arr = selected
                    .iter()
                    .map(|i| options[*i].clone())
                    .collect::<Vec<String>>();

                value = serde_json::to_value(arr).unwrap();
            }
            "password" => {
                let confirm_prompt = "确认密码";
                let mismatch_err = "两次输入的密码不一致";
                value = Value::String(
                    Password::with_theme(&ColorfulTheme::default())
                        .with_prompt(description)
                        .with_confirmation(confirm_prompt, mismatch_err)
                        .interact()
                        .unwrap(),
                );
            }
            _ => {}
        }

        if !(arg_type == "password") {
            println!(
                "{}: {}",
                description,
                serde_json::to_string(&value).unwrap().green()
            );
        }

        ret.insert(arg.name.clone(), value);
    }
    serde_json::to_value(ret).unwrap()
}
