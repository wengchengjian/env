use crate::{get_temp_dir, Result};
use anyhow::anyhow;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Client};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn create_pbr(size: usize) -> ProgressBar {
    let pb = ProgressBar::new(size as u64);
    
    pb.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
    .unwrap()
    .progress_chars("#>-"));
    
    pb
}

pub async fn download_packages(url: &str) -> Result<String> {
    let url_last = url.split("/").last().unwrap();
    let base_dir = get_temp_dir().join("env_download_cache");

    // 创建缓存目录如果不存在
    if !base_dir.exists() {
        fs::create_dir_all(&base_dir)?;
    }

    let filename = base_dir.join(url_last).to_str().unwrap().to_string();
    let path = Path::new(&filename);
    println!("下载包 {} 到 {}", url, filename);

    let client = Client::new();
    let total_size = {
        let resp = client.head(url).send().await?;
        if resp.status().is_success() {
            resp.headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse().ok())
                .unwrap_or(0)
        } else {
            return Err(anyhow!(
                "Couldn't download URL: {}. Error: {:?}",
                url,
                resp.status(),
            ));
        }
    };
    let mut request = client.get(url);
    
    let mut has_size = 0;
    if path.exists() {
        has_size = path.metadata()?.len().saturating_sub(1);
        request = request.header(header::RANGE, format!("bytes={}-", has_size));
    }
    let pb = create_pbr(total_size as usize - has_size as usize);

    let mut source = request.send().await?;
    let mut dest = OpenOptions::new().create(true).append(true).open(&path)?;
    while let Some(chunk) = source.chunk().await? {
        dest.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_with_message("Download complete");

    Ok(filename)
}

// 复制文件到指定目录, 并返回复制后的文件位置
pub fn copy_file_to_dir(source_file_path: &str, destination_dir_path: &str) -> io::Result<String> {
    let source_file = Path::new(source_file_path);
    let destination_dir = Path::new(destination_dir_path);

    // 确保源文件存在
    if !source_file.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "源文件不存在"));
    }

    // 确保目标目录存在，如果不存在则创建
    if !destination_dir.exists() {
        fs::create_dir_all(destination_dir)?;
    }

    // 构建目标文件的完整路径
    let destination_file = PathBuf::from(destination_dir).join(source_file.file_name().unwrap());
    
    let ret = destination_file.to_str().unwrap().to_string();
    
    fs::copy(source_file, &destination_file).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("复制文件失败: {}", e)
        )
    })?;
    // 删除临时文件
    fs::remove_file(source_file_path).unwrap();
    Ok(ret)
}