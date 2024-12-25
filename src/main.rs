use std::path::PathBuf;
use clap::Parser;
use env::env_config::EnvConfig;
use env::environment::environments::flush_env_config;
use env::{choose_and_install_from, choose_version, get_env_home_dir, EnvArgs, EnvSubCommand, Result};
use env::{install_all, install_environment, choose_and_install};

#[tokio::main]
async fn main() -> Result<()> {
    let args = EnvArgs::parse();

    if let Some(command) = args.command {
        match command {
            EnvSubCommand::Dev(args) => {
                let env_config = EnvConfig::load_deserialize();

                let install_dir = PathBuf::from(&env_config.install_path);

                if args.all {
                    install_all(&install_dir).await?;
                } else if let Some(env) = args.name {
                    choose_and_install_from(&env, &install_dir).await?;
                } else {
                    choose_and_install(&install_dir).await?;
                }
            }
            EnvSubCommand::Choose { name } => {
                choose_version(&name)?;
            },
            EnvSubCommand::Config { dir,flush} => {
                let mut env_config = EnvConfig::load_deserialize();
                
                if let Some(dir) = dir {
                    env_config.install_path = dir.as_os_str().to_str().unwrap().to_string();
                }

                if flush {
                    flush_env_config(&mut env_config);
                }
                env_config.save();
                // 打印配置
                println!("当前配置: \n{}", serde_json::to_string_pretty(&env_config).unwrap());
            }
        }
    } else {
        // 打印help
    }

    
    Ok(())
}
