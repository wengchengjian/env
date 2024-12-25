use std::fs::OpenOptions;
use std::{env, io};
use std::fmt::format;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;

pub const DEFAULT_JDK_VERSION: &str = "17.0.12";


pub fn set_persistent_path(new_path: &str) -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let path = format!("%Path%;{new_path}");
        Command::new("setx")
            .args(&["Path", &path])
            .output()?;
    } else {
        set_persistent_path_unix(new_path, false)?
    }
   
    Ok(())
}

pub fn set_persistent_path_unix(new_path: &str, system_level: bool) -> io::Result<()> {
    let config_file = if system_level {
        PathBuf::from("/etc/environment")
    } else {
        let home = env::var("HOME").unwrap();
        PathBuf::from(home + "/.bashrc")
    };

    let mut lines = Vec::new();
    if config_file.exists() {
        let file = OpenOptions::new().read(true).open(&config_file)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            lines.push(line?);
        }
    }


    let mut found = false;
    let mut updated_lines = Vec::new();
    for line in lines {
        if line.contains("PATH=") {
            found = true;
            let mut parts: Vec<&str> = line.split('=').collect();
            let mut paths: Vec<&str> = parts[1].split(':').collect();
            if!paths.contains(&new_path) {
                paths.push(new_path);
                let new_line = format!("export PATH={}\n", paths.join(":"));
                updated_lines.push(new_line);
            } else {
                updated_lines.push(line);
            }
        } else {
            updated_lines.push(line);
        }
    }


    if!found {
        updated_lines.push(format!("export PATH=$PATH:{}\n", new_path));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&config_file)?;
    file.write_all(updated_lines.join("\n").as_bytes())?;
    Ok(())
}



pub fn set_persistent_env(var_name: &str, var_value: &str) -> crate::Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("setx")
            .args(&[var_name, var_value])
            .output()
            .map_err(|e| format!("Failed to execute setx command: {}", e))?;
    } else {
        set_persistent_env_unix(var_name, var_value, false)?;
    }
    Ok(())

}

pub fn set_persistent_env_unix(var_name: &str, var_value: &str, system_level: bool) -> io::Result<()> {
    let config_file = if system_level {
        PathBuf::from("/etc/environment")
    } else {
        let home = env::var("HOME").unwrap();
        PathBuf::from(home + "/.bashrc")
    };


    let entry = format!("\nexport {}={}\n", var_name, var_value);
    let mut lines = Vec::new();
    if config_file.exists() {
        let file = OpenOptions::new().read(true).open(&config_file)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            lines.push(line?);
        }
    }

    let var_like = &format!("export {}", var_name);

    let mut indexs = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.contains(var_like) {
            indexs.push(index);
        }
    }
    // 移除重复元素
    indexs.iter().for_each(| i | {
        lines.remove(*i);
    });
    lines.push(entry);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&config_file)?;
    
    file.write_all(lines.join("\n").as_bytes())?;

    Ok(())
}