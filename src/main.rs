use anyhow::Result;
use clap::Parser;
use env::env_config::{flush_env_config, EnvConfig};
use env::install::{choose_and_install, choose_and_install_from, choose_version};
use env::{EnvArgs, EnvSubCommand};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let args = EnvArgs::parse();

    if let Err(e) = handle_cmd(&args).await {
        println!("env error: {}", e);
    }
    
    Ok(())
}

pub async fn handle_cmd(args: &EnvArgs) -> Result<()> {

    let mut env_config = EnvConfig::load_deserialize()?;

    if let Some(command) = &args.command {
        match command {
            EnvSubCommand::Dev(args) => {
                let install_dir = PathBuf::from(&env_config.install_path);

                if let Some(name) = &args.name {
                    let env = env_config.get_enviroment(name);
                    
                    if env.is_none() {
                        println!("不支持的环境: {}", name);
                        return Ok(());
                    }

                    choose_and_install_from(&env.unwrap(), &install_dir).await?;
                } else {
                    choose_and_install(&install_dir).await?;
                }
            }
            EnvSubCommand::Choose { name } => {
                choose_version(name)?;
            },
            EnvSubCommand::Config { dir,flush} => {
                
                
                if let Some(dir) = dir {
                    env_config.install_path = dir.as_os_str().to_str().unwrap().to_string();
                    EnvConfig::save(&env_config)?;
                }
                
                if *flush {
                    flush_env_config()?;
                }

                println!("\n{}", serde_json::to_string_pretty(&env_config)?);
            }
        }
    }
    Ok(())

}
