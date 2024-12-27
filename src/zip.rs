use anyhow::{anyhow, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use serde_json::{json, Value};
use sevenz_rust::decompress_file;
use std::fs::{self, File};
use std::io::BufReader;
use std::io::{copy, Read};
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::download::create_pbr;

lazy_static! {
    pub static ref DEFAULT_FORMAT: Value = json!({
        "windows": "zip",
        "macos": "tar.gz",
        "linux": "tar.gz"
    });
}

pub fn auto_unzip(filename: &str, output: &str) -> Result<()> {
    let file_path = Path::new(filename);
    let output_dir = Path::new(output);

    if let Err(e) = fs::create_dir_all(output_dir) {
        return Err(anyhow!("无法创建输出目录: {}", e));
    }

    if let Some(file_type) = get_file_type(file_path) {
        match file_type {
            FileType::ZIP => unzip_file(file_path, output_dir)?,
            FileType::GZ => ungzip_file(file_path, output_dir)?,
            FileType::TAR => untar_file(file_path, output_dir)?,
            FileType::BZ2 => unbzip2_file(file_path, output_dir)?,
            FileType::XZ => unxz_file(file_path, output_dir)?,
            FileType::SZ => un7z_file(file_path, output_dir)?,
            FileType::TARGZ => untargz_file(file_path, output_dir)?,
            FileType::UNKNOWN => return Err(anyhow!("不支持的压缩格式")),
        }
    } else {
        return Err(anyhow!("无法识别文件类型"));
    }
    fs::remove_file(filename)?;
    Ok(())
}

fn get_file_type(file_path: &Path) -> Option<FileType> {
    let filename = file_path.file_name().unwrap().to_str().unwrap();
    if filename.contains(".tar.gz") {
        return Some(FileType::TARGZ);
    }

    if let Some(ext) = file_path.extension() {
        match ext.to_str() {
            Some("zip") => Some(FileType::ZIP),
            Some("gz") => Some(FileType::GZ),
            Some("tar") => Some(FileType::TAR),
            Some("bz2") => Some(FileType::BZ2),
            Some("xz") => Some(FileType::XZ),
            Some("7z") => Some(FileType::SZ),
            _ => None,
        }
    } else {
        // 检查文件头
        if let Ok(mut file) = File::open(file_path) {
            let mut buffer = [0; 6];
            if file.read_exact(&mut buffer).is_ok() {
                match &buffer {
                    [0x50, 0x4b, 0x03, 0x04, ..] => Some(FileType::ZIP),
                    [0x1f, 0x8b, ..] => Some(FileType::GZ),
                    [0x42, 0x5a, 0x68, ..] => Some(FileType::BZ2),
                    [0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00] => Some(FileType::XZ),
                    [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c] => Some(FileType::SZ),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn create_unzip_progress_bar(total: usize) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap());
    pb
}

fn unzip_file(file_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(file_path).map_err(|e| anyhow!("无法打开 zip 文件: {}", e))?;

    let mut archive = ZipArchive::new(file).map_err(|e| anyhow!("无法打开 zip 存档: {}", e))?;
    let total_files = archive.len();
    let pb = create_unzip_progress_bar(total_files);
    for i in 0..total_files {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| anyhow!("无法读取 zip 条目 {}: {}", i, e))?;

        let entry_path = output_dir.join(entry.mangled_name());
        if entry.is_dir() {
            fs::create_dir_all(&entry_path)
                .map_err(|e| anyhow!("无法创建目录 {}: {}", entry_path.display(), e))?;
        } else {
            if let Some(parent) = entry_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(&entry_path)?;

            copy(&mut entry, &mut file)
                .map_err(|e| anyhow!("无法解压文件 {}: {}", entry_path.display(), e))?;
        }
        pb.inc(1);
    }
    pb.finish_with_message("解压完成");

    Ok(())
}

fn ungzip_file(file_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(file_path)?;
    let decoder = GzDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let output_file_path = output_dir.join(file_path.file_stem().unwrap());
    let mut output_file = File::create(&output_file_path)?;

    copy(&mut reader, &mut output_file)?;
    Ok(())
}

fn untar_file(file_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(file_path)?;
    let mut archive = Archive::new(file);
    archive.unpack(output_dir)?;
    Ok(())
}

fn unbzip2_file(file_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(file_path)?;
    let decoder = BzDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let output_file_path = output_dir.join(file_path.file_stem().unwrap());
    let mut output_file = File::create(&output_file_path)?;

    copy(&mut reader, &mut output_file)?;
    Ok(())
}

fn unxz_file(file_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(file_path)?;
    let decoder = XzDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let output_file_path = output_dir.join(file_path.file_stem().unwrap());
    let mut output_file = File::create(&output_file_path)?;

    copy(&mut reader, &mut output_file)?;
    Ok(())
}

fn un7z_file(file_path: &Path, output_dir: &Path) -> Result<()> {
    decompress_file(file_path, output_dir)?;
    Ok(())
}

fn untargz_file(file_path: &Path, output_dir: &Path) -> Result<()> {
    if let Err(e) = fs::create_dir_all(output_dir) {
        return Err(anyhow!("无法创建输出目录: {}", e));
    }

    let file = File::open(file_path)?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut archive = Archive::new(reader);
    let total_files = archive.entries()?.count();
    let pb = create_unzip_progress_bar(total_files);
    for file in archive.entries()? {
        let mut file = file?;
        let path = file.path()?;

        if path.as_os_str().is_empty() {
            return Err(anyhow!("无效的 tar 条目路径"));
        }

        let output_path = output_dir.join(&path);
        if file.header().entry_type().is_dir() {
            fs::create_dir_all(&output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut out_file = File::create(&output_path)?;
            let filepath = file.header().path()?;
            let filename = filepath.file_name().unwrap().to_str().unwrap();

            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                filename,
                output_path.display(),
                file.size()
            );
            copy(&mut file, &mut out_file)?;
        }
        pb.inc(1);
    }

    pb.finish_with_message("解压完成");

    Ok(())
}

#[derive(PartialEq, Eq)]
enum FileType {
    ZIP,
    GZ,
    TAR,
    BZ2,
    XZ,
    SZ,
    UNKNOWN,
    TARGZ,
}

#[cfg(test)]
mod tests {
    use crate::zip::auto_unzip;

    #[test]
    fn test_auto_unzip() {
        //zip
        auto_unzip(
            "E:\\wengchengjian\\下载\\jdk-17.0.12_windows-x64_bin.zip",
            "E:\\project\\rust-project\\env\\test\\java",
        )
        .unwrap();
        //tar.gz
        auto_unzip(
            "E:\\wengchengjian\\下载\\jdk-17.0.12_linux-aarch64_bin.tar.gz",
            "E:\\project\\rust-project\\env\\test\\java2",
        )
        .unwrap();
    }
}
