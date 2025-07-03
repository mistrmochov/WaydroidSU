use crate::constants::WAYDROID_CONFIG;
use crate::container::{WaydroidContainer, has_overlay};
use crate::msg_sub;
use crate::{msg_err, msg_err_str};
use anyhow::anyhow;
use colored::*;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use indicatif::{ProgressBar, ProgressStyle};
use ini::configparser::ini::Ini;
use rand::{Rng, distr::Alphanumeric};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_LENGTH;
use serde::Deserialize;
use std::env::temp_dir;
use std::fs::File;
use std::fs::{self, Permissions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::result::Result::Ok as OtherOk;
use std::thread::sleep;
use std::time::Duration;
use zip::read::ZipArchive;

#[derive(Debug, Deserialize)]
struct MagiskInfo {
    version: String,
    link: String,
}

#[derive(Debug, Deserialize)]
struct FullJson {
    magisk: MagiskInfo,
}

pub fn unzip_file(zip_path: &str, extract_to: &str) -> anyhow::Result<()> {
    let file = File::open(zip_path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = Path::new(extract_to).join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut outfile = File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

pub fn get_magisk_info(json_str: &str) -> anyhow::Result<(String, String)> {
    let parsed: FullJson = serde_json::from_str(json_str)?;
    Ok((parsed.magisk.version, parsed.magisk.link))
}

fn get_systemimg_path() -> anyhow::Result<PathBuf, Box<dyn std::error::Error>> {
    let mut conf = Ini::new();
    conf.load(WAYDROID_CONFIG)?;
    let images = conf
        .get("waydroid", "images_path")
        .expect(&msg_err_str("Coldn't get path for images!"));
    Ok(PathBuf::from(images).join("system.img"))
}

pub fn mount_system(
    mut waydroid: WaydroidContainer,
    resize: bool,
    quiet: bool,
) -> anyhow::Result<bool> {
    if waydroid.is_session_running(true, false)? {
        waydroid.stop(true)?;
    }
    let system = get_systemimg_path().expect(&msg_err_str("Failed to get system img path!"));
    if !system.exists() {
        return Err(anyhow!("Couldn't find Waydroid system image!"));
    }
    if let Ok(status) = Command::new("e2fsck")
        .args(["-y", "-f", system.to_string_lossy().trim()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        if !status.success() {
            return Err(anyhow!("Command e2fsck exited with failure code!"));
        }
    } else {
        return Err(anyhow!("Failed to run e2fsck command!"));
    }

    if resize {
        if !quiet {
            msg_sub("Resizing system image");
        }
        if let Ok(status) = Command::new("resize2fs")
            .args([system.to_string_lossy().trim(), "3G"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            if !status.success() {
                return Err(anyhow!("Command resize2fs exited with failure code!"));
            }
        } else {
            return Err(anyhow!("Failed to run resize2fs command!"));
        }
    }

    let mnt = temp_dir().join("waydroidsu/mnt");
    if !mnt.exists() {
        fs::create_dir_all(mnt.clone())?;
    }
    if !quiet {
        msg_sub("Mounting system image");
    }
    let tries = 5;
    for i in 1..=tries {
        if let Ok(status) = Command::new("mount")
            .args([
                "-o",
                "rw,loop",
                system.to_string_lossy().trim(),
                mnt.to_string_lossy().trim(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            if !status.success() {
                if i == tries {
                    return Err(anyhow!("Command mount exited with failure code!"));
                }
            } else {
                break;
            }
        } else {
            if i == tries {
                return Err(anyhow!("Failed to run mount command!"));
            }
        }
        sleep(Duration::from_secs(1));
    }
    Ok(true)
}

pub fn umount_system(quiet: bool) -> anyhow::Result<bool> {
    let mnt = temp_dir().join("waydroidsu/mnt");
    if !quiet {
        msg_sub("Umounting system image");
    }
    let tries = 5;
    for i in 1..=tries {
        if let Ok(status) = Command::new("umount").arg(mnt.clone()).status() {
            if !status.success() {
                if i == tries {
                    return Err(anyhow!("Command umount exited with failure code!"));
                }
            } else {
                break;
            }
        } else {
            if i == tries {
                return Err(anyhow!("Failed to run umount command!"));
            }
        }
        sleep(Duration::from_secs(1));
    }
    Ok(true)
}

pub fn is_mounted_at(target_mount: &str) -> std::io::Result<bool> {
    let file = File::open("/proc/mounts")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == target_mount {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub fn download_file(url: &str, output_path: &str, quiet: bool) -> anyhow::Result<()> {
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

    let response = client.get(url).send()?;

    let total_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let pb = if !quiet {
        let bar = ProgressBar::new(total_size);
        bar.set_style(
            ProgressStyle::with_template(" [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(bar)
    } else {
        None
    };

    let mut source = response;
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    let mut downloaded = 0u64;
    let mut buffer = [0; 8192];

    while let Ok(n) = source.read(&mut buffer) {
        if n == 0 {
            break;
        }
        writer.write_all(&buffer[..n])?;
        downloaded += n as u64;
        if let Some(pb) = &pb {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = pb {
        pb.finish();
    }

    Ok(())
}

pub fn get_arch() -> (&'static str, bool) {
    let arch = std::env::consts::ARCH;
    if arch == "x86_64" {
        return ("x86_64", true);
    } else if arch == "aarch64" {
        return ("arm64-v8a", true);
    } else {
        msg_err(&format!("{} {}", arch.bold(), "isn't supported!"));
        return (arch, false);
    }
}

pub fn create_dir_check(dir: PathBuf, erasing: bool) -> anyhow::Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir.clone())?;
    } else {
        if erasing {
            fs::remove_dir_all(dir.clone())?;
            fs::create_dir_all(dir)?;
        }
    }
    Ok(())
}

pub fn chmod_755_recursive(dir: PathBuf) -> anyhow::Result<()> {
    fs::set_permissions(dir.clone(), Permissions::from_mode(0o755))?;
    if let OtherOk(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let OtherOk(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    chmod_755_recursive(path)?;
                } else {
                    fs::set_permissions(path, Permissions::from_mode(0o755))?;
                }
            }
        }
    }
    Ok(())
}

pub fn cp_dir(source: PathBuf, destination: PathBuf) -> anyhow::Result<()> {
    if let Some(name) = source.file_name() {
        let dest = destination.join(name);
        if !dest.exists() {
            fs::create_dir(&dest)?;
        }

        for entry in fs::read_dir(&source)? {
            let item = entry?;
            let item_path = item.path();

            if let Some(item_name) = item_path.file_name() {
                let target_path = dest.join(item_name);

                if item_path.is_dir() {
                    cp_dir(item_path.clone(), dest.clone())?;
                } else {
                    if item_path.exists() {
                        if target_path.exists() {
                            fs::remove_file(&target_path)?;
                        }
                        fs::copy(&item_path, &target_path)?;
                    } else {
                        msg_err(&format!(
                            "Warning: File \"{}\" doesn't exist or was moved.",
                            item_path.to_string_lossy()
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn gzip_compress(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);

    let output = File::create(output_path)?;
    let writer = BufWriter::new(output);

    let mut encoder = GzEncoder::new(writer, Compression::default());
    std::io::copy(&mut reader.take(u64::MAX), &mut encoder)?;
    encoder.finish()?; // flush + close

    Ok(())
}

pub fn gzip_decompress(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let input = File::open(input_path)?;
    let reader = GzDecoder::new(BufReader::new(input));

    let output = File::create(output_path)?;
    let mut writer = BufWriter::new(output);

    std::io::copy(&mut reader.take(u64::MAX), &mut writer)?;

    Ok(())
}

pub fn generate_random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn getenforce() -> anyhow::Result<bool> {
    let output;
    if let Ok(out) = Command::new("getenforce").output() {
        output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    } else {
        output = String::new();
    }

    Ok(output == "Enforcing")
}

pub fn create_tmpdir() -> anyhow::Result<()> {
    let has_overlay = has_overlay().expect(&msg_err_str(
        "Couldn't reach the \"mount_overlays\" config.",
    ));
    let tempdir = temp_dir().join("waydroidsu");
    if tempdir.exists() {
        if has_overlay && is_mounted_at("mnt")? {
            umount_system(true)?;
        }
        fs::remove_dir_all(&tempdir)?;
    }
    fs::create_dir(&tempdir)?;
    Ok(())
}
