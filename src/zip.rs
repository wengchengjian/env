use std::fs::{self, File};
use std::io::copy;
use std::io::BufReader;
use std::path::Path;
use zip::ZipArchive;
use flate2::read::GzDecoder;
use tar::Archive;
use bzip2::read::BzDecoder;
use xz2::read::XzDecoder;
use std::io::Read;
use sevenz_rust::decompress_file;


pub fn auto_unzip(filename: &str, output: &str) {
    let file_path = Path::new(filename);
    let output_dir = Path::new(output);

    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("无法创建输出目录: {}", e);
        return;
    }

    if let Some(file_type) = get_file_type(file_path) {
        match file_type {
            FileType::ZIP => unzip_file(file_path, output_dir),
            FileType::GZ => ungzip_file(file_path, output_dir),
            FileType::TAR => untar_file(file_path, output_dir),
            FileType::BZ2 => unbzip2_file(file_path, output_dir),
            FileType::XZ => unxz_file(file_path, output_dir),
            FileType::SZ => un7z_file(file_path, output_dir),
            FileType::TARGZ => untargz_file(file_path, output_dir),
            FileType::UNKNOWN => eprintln!("不支持的压缩格式"),
        }
    } else {
        eprintln!("无法识别文件类型");
    }
    fs::remove_file(filename).unwrap();
}



fn get_file_type(file_path: &Path) -> Option<FileType> {
    let filename = file_path.file_name().unwrap().to_str().unwrap();
    if filename.contains(".tar.gz") {
        return Some(FileType::TARGZ)
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
                    [0x50, 0x4b, 0x03, 0x04,..] => Some(FileType::ZIP),
                    [0x1f, 0x8b,..] => Some(FileType::GZ),
                    [0x42, 0x5a, 0x68,..] => Some(FileType::BZ2),
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

fn untargz_file(file_path: &Path, output_dir: &Path) {
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("无法创建输出目录: {}", e);
        return;
    }


    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法打开 gzip 文件: {}", e);
            return;
        }
    };


    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    let mut archive = Archive::new(reader);

    for file in archive.entries().unwrap() {
        let mut file = match file {
            Ok(file) => file,
            Err(e) => {
                eprintln!("无法读取 tar 条目: {}", e);
                continue;
            }
        };

        let path = match file.path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("无法获取 tar 条目路径: {}", e);
                continue;
            }
        };
        if path.as_os_str().is_empty() {
            eprintln!("无效的 tar 条目路径");
            continue;
        }

        let output_path = output_dir.join(path);
        if file.header().entry_type().is_dir() {
            if let Err(e) = fs::create_dir_all(&output_path) {
                eprintln!("无法创建目录 {}: {}", output_path.display(), e);
                continue;
            }
        } else {
            if let Some(parent) = output_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("无法创建父目录 {}: {}", parent.display(), e);
                    continue;
                }
            }
            
            let mut out_file = match File::create(&output_path) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("无法创建文件 {}: {}", output_path.display(), e);
                    continue;
                }
            };
            let filepath = file.header().path().unwrap();
            let filename = filepath.file_name().unwrap().to_str().unwrap();
            
            println!("File {} extracted to \"{}\" ({} bytes)",filename, output_path.display(), file.size());

            if let Err(e) = copy(&mut file, &mut out_file) {
                eprintln!("无法将内容复制到文件 {}: {}", output_path.display(), e);
                continue;
            }
        }
    }
}

fn unzip_file(file_path: &Path, output_dir: &Path) {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法打开 zip 文件: {}", e);
            return;
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            eprintln!("无法打开 zip 存档: {}", e);
            return;
        }
    };

    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("无法读取 zip 条目 {}: {}", i, e);
                continue;
            }
        };
        let entry_path = output_dir.join(entry.mangled_name());
        if entry.is_dir() {
            if let Err(e) = fs::create_dir_all(&entry_path) {
                eprintln!("无法创建目录 {}: {}", entry_path.display(), e);
            }
        } else {
            if let Some(parent) = entry_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("无法创建父目录 {}: {}", parent.display(), e);
                    continue;
                }
            }
            let mut file = match File::create(&entry_path) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("无法创建文件 {}: {}", entry_path.display(), e);
                    continue;
                }
            };
            println!("File {} extracted to \"{}\" ({} bytes)",entry.name(), entry_path.display(), entry.size());
            
            if let Err(e) = copy(&mut entry, &mut file) {
                eprintln!("无法解压文件 {}: {}", entry_path.display(), e);
            }
        }
    }
}


fn ungzip_file(file_path: &Path, output_dir: &Path) {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法打开 gzip 文件: {}", e);
            return;
        }
    };


    let decoder = GzDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let output_file_path = output_dir.join(file_path.file_stem().unwrap());
    let mut output_file = match File::create(&output_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法创建输出文件: {}", e);
            return;
        }
    };


    if let Err(e) = copy(&mut reader, &mut output_file) {
        eprintln!("无法解压文件: {}", e);
    }
}


fn untar_file(file_path: &Path, output_dir: &Path) {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法打开 tar 文件: {}", e);
            return;
        }
    };
    
    let mut archive = Archive::new(file);
    if let Err(e) = archive.unpack(output_dir) {
        eprintln!("无法解压 tar 文件: {}", e);
    }
}


fn unbzip2_file(file_path: &Path, output_dir: &Path) {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法打开 bz2 文件: {}", e);
            return;
        }
    };


    let mut decoder = BzDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let output_file_path = output_dir.join(file_path.file_stem().unwrap());
    let mut output_file = match File::create(&output_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法创建输出文件: {}", e);
            return;
        }
    };


    if let Err(e) = copy(&mut reader, &mut output_file) {
        eprintln!("无法解压文件: {}", e);
    }
}


fn unxz_file(file_path: &Path, output_dir: &Path) {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法打开 xz 文件: {}", e);
            return;
        }
    };


    let decoder = XzDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let output_file_path = output_dir.join(file_path.file_stem().unwrap());
    let mut output_file = match File::create(&output_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法创建输出文件: {}", e);
            return;
        }
    };


    if let Err(e) = copy(&mut reader, &mut output_file) {
        eprintln!("无法解压文件: {}", e);
    }
}


fn un7z_file(file_path: &Path, output_dir: &Path) {
    if let Err(e) = decompress_file(file_path, output_dir) {
        eprintln!("无法解压 7z 文件: {}", e);
    }
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
        auto_unzip("E:\\wengchengjian\\下载\\jdk-17.0.12_windows-x64_bin.zip", "E:\\project\\rust-project\\env\\test\\java");
        //tar.gz
        auto_unzip("E:\\wengchengjian\\下载\\jdk-17.0.12_linux-aarch64_bin.tar.gz", "E:\\project\\rust-project\\env\\test\\java2");
    }
}