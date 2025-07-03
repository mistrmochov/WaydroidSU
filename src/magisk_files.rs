use crate::constants::*;
use crate::container::{WaydroidContainer, has_overlay};
use crate::magisk::Magisk;
use crate::utils::{
    chmod_755_recursive, cp_dir, create_dir_check, download_file, generate_random_string,
    get_magisk_info, getenforce, gzip_compress, gzip_decompress, is_mounted_at, mount_system,
    umount_system, unzip_file,
};
use crate::{get_data_home, msg_end, msg_err, msg_err_str, msg_main, msg_regular, msg_sub};
use anyhow::{Ok, anyhow};
use colored::*;
use std::env;
use std::env::temp_dir;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::result::Result::Ok as OtherOk;

pub fn magisk_is_installed() -> anyhow::Result<bool> {
    let mut waydroid = WaydroidContainer::new()?;
    let magisk_dir;
    if waydroid.is_session_running(true, true)? {
        magisk_dir = PathBuf::from(WAYDROID_DIR).join("rootfs/system/etc/init/magisk");
    } else if has_overlay().expect(&msg_err_str("Failed to get overlay result!")) {
        magisk_dir = PathBuf::from(WAYDROID_DIR).join("overlay/system/etc/init/magisk");
    } else {
        mount_system(waydroid, false, true)?;
        magisk_dir = PathBuf::from("/mnt/waydroid").join("system/etc/init/magisk");
        let magisk_dir_result = magisk_dir.exists();
        umount_system(true)?;
        return Ok(magisk_dir_result);
    }
    Ok(magisk_dir.exists())
}

pub fn magisk_is_set_up() -> anyhow::Result<bool> {
    let magisk64_init;
    if has_overlay().expect(&msg_err_str("Failed to get \"mount_overlays\" config,")) {
        magisk64_init = PathBuf::from(WAYDROID_DIR)
            .join("overlay")
            .join(MAGISK_DIR)
            .join("magisk64");
    } else {
        magisk64_init = PathBuf::from(WAYDROID_DIR)
            .join("rootfs")
            .join(MAGISK_DIR)
            .join("magisk64");
    }
    let magisk64_data = PathBuf::from(get_data_home()?).join("waydroid/data/adb/magisk/magisk64");
    Ok(magisk64_init.exists() && magisk64_data.exists())
}

pub fn waydroid_su(args: Vec<&str>) -> anyhow::Result<(bool, String)> {
    let mut waydroid = WaydroidContainer::new()?;
    let selinux_enforcing = getenforce()?;
    if !waydroid.is_container_running()? {
        return Err(anyhow!("Waydroid container isn't running!"));
    }

    if !magisk_is_installed()? {
        return Err(anyhow!("Magisk is not installed!"));
    }

    if args.is_empty() {
        return Err(anyhow!("su arguments are empty"));
    }
    let lxc = PathBuf::from(WAYDROID_DIR).join("lxc");
    let path_var = env::var("PATH")?;
    let mut cmd = Command::new("lxc-attach");
    if selinux_enforcing {
        cmd.args(["-P", lxc.to_string_lossy().trim(), "-n", "waydroid", "--"])
            .env("PATH", path_var + ":/system/bin:/vendor/bin");
    } else {
        cmd.args([
            "-P",
            lxc.to_string_lossy().trim(),
            "-n",
            "waydroid",
            "--",
            "su",
            "-c",
        ])
        .env("PATH", path_var + ":/system/bin:/vendor/bin");
    }

    for arg in args {
        cmd.arg(arg);
    }

    let out = cmd.output()?;

    let success;
    if !out.status.success() {
        success = false;
        let error;
        if out.stderr.is_empty() {
            error = String::from_utf8_lossy(&out.stdout);
        } else {
            error = String::from_utf8_lossy(&out.stderr);
        }
        msg_err(error.trim());
    } else {
        success = true;
    }
    let out_send;
    if out.stdout.is_empty() {
        out_send = String::from_utf8_lossy(&out.stderr);
    } else {
        out_send = String::from_utf8_lossy(&out.stdout);
    }
    Ok((success, out_send.trim().to_string()))
}

pub fn install(arch: &str, custom_apk: &str, update: bool) -> anyhow::Result<()> {
    if !magisk_is_installed()? || update {
        let mut waydroid = WaydroidContainer::new()?;
        let has_overlay = has_overlay().expect(&msg_err_str(
            "Couldn't reach the \"mount_overlays\" config.",
        ));

        if !has_overlay {
            print!(
                "[{}] {} {} ",
                "WSU".blue().bold(),
                "Your setup has \"mount_overlays\" disabled, do you wish to modify system image?"
                    .bold(),
                "[Y/n]".blue().bold()
            );
            io::stdout().flush()?; // Ensure the prompt is displayed immediately

            let mut input = String::new();
            io::stdin().read_line(&mut input)?; // Read user input
            println!();

            let input = input.trim(); // Remove trailing newline and spaces
            if !input.eq_ignore_ascii_case("y") {
                return Ok(());
            }
        }

        if !update {
            msg_main("Installing KitsuneMagisk...");
        }

        if waydroid.is_session_running(true, true)? && !update {
            msg_sub("Stopping Waydroid session");
            waydroid.stop(true)?;
        }

        let tempdir = temp_dir().join("waydroidsu");
        let magisk_tmp = tempdir.join("magisk");
        if !tempdir.exists() {
            fs::create_dir(&tempdir)?;
        }
        let apk = tempdir.join("magisk.apk");
        if custom_apk == "" {
            let json_file = tempdir.join("canary.json");
            download_file(
                "https://raw.githubusercontent.com/mistrmochov/KitsuneMagisk-Waydroid/refs/heads/kitsune/canary.json",
                &json_file.to_string_lossy(),
                true,
            )?;
            let json_data = fs::read_to_string(json_file)?;
            let (version, link) = get_magisk_info(&json_data)?;
            msg_sub(&format!(
                "{} {} {}",
                "Downloading",
                "KitsuneMagisk:",
                version.blue().bold()
            ));
            download_file(&link, &apk.to_string_lossy(), false)?;
        } else {
            if PathBuf::from(custom_apk).exists() && PathBuf::from(custom_apk).is_file() {
                fs::copy(PathBuf::from(custom_apk), apk.clone())?;
            } else {
                return Err(anyhow!("Custom apk file doesn't exist or isn't file."));
            }
        }
        if !magisk_tmp.exists() {
            fs::create_dir(magisk_tmp.clone())?;
        }
        msg_sub("Extracting KitsuneMagisk");
        unzip_file(&apk.to_string_lossy(), &magisk_tmp.to_string_lossy())?;
        let libs = magisk_tmp.join("lib").join(arch);
        let assets = magisk_tmp.join("assets");
        let libs32;
        if arch == "x86_64" {
            libs32 = magisk_tmp.join("lib").join("x86");
        } else {
            libs32 = magisk_tmp.join("lib").join("armeabi-v7a");
        }
        if !libs.exists() || !libs32.exists() || !assets.exists() {
            return Err(anyhow!("Structure of apk file hasn't been recognized."));
        }

        let rootfs;
        let overlay_rw = PathBuf::from(WAYDROID_DIR).join("overlay_rw/system");
        if has_overlay {
            rootfs = PathBuf::from(WAYDROID_DIR).join("overlay");
        } else {
            if !is_mounted_at(&tempdir.join("mnt").to_string_lossy())? {
                mount_system(waydroid.clone(), true, false)?;
            }
            rootfs = tempdir.join("mnt");
        }
        let magisk_dir = rootfs.join(MAGISK_DIR);
        let bootanim_rc_path = rootfs.join(BOOTANIM_RC_PATH);
        let booatnim_rc_gz_path = rootfs.join(BOOTANIM_RC_GZ_PATH);

        create_dir_check(magisk_dir.clone(), true)?;
        create_dir_check(rootfs.join("system/addon.d"), has_overlay)?;
        create_dir_check(rootfs.join("sbin"), true)?;
        if overlay_rw.join(MAGISK_DIR).exists() {
            fs::remove_dir_all(overlay_rw.join(MAGISK_DIR))?;
        }
        if overlay_rw.join(BOOTANIM_RC_PATH).exists() {
            fs::remove_file(overlay_rw.join(BOOTANIM_RC_PATH))?;
        }
        if overlay_rw.join(BOOTANIM_RC_GZ_PATH).exists() {
            fs::remove_file(overlay_rw.join(BOOTANIM_RC_GZ_PATH))?;
        }
        if overlay_rw.join("system/addon.d/99-magisk.sh").exists() {
            fs::remove_file(overlay_rw.join("system/addon.d/99-magisk.sh"))?;
        }

        msg_sub("Copying scripts and binaries");
        let entries = fs::read_dir(libs)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.starts_with("lib") && file_name.ends_with(".so") {
                    let target_name = file_name
                        .trim_start_matches("lib")
                        .trim_end_matches(".so")
                        .to_string();
                    fs::copy(path, magisk_dir.join(target_name))?;
                }
            }
        }
        fs::copy(libs32.join("libmagisk32.so"), magisk_dir.join("magisk32"))?;

        let entries = fs::read_dir(assets.clone())?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Some(file_name) = path.clone().file_name() {
                if file_name == "boot_patch.sh"
                    || file_name == "util_functions.sh"
                    || file_name == "addon.d.sh"
                {
                    fs::copy(path, magisk_dir.join(file_name))?;
                }
            }
        }
        chmod_755_recursive(magisk_dir.clone())?;
        cp_dir(assets.join("chromeos"), magisk_dir.clone())?;
        chmod_755_recursive(magisk_dir.join("chromeos"))?;
        fs::copy(assets.join("stub.apk"), magisk_dir.join("stub.apk"))?;
        fs::copy(
            apk.clone(),
            magisk_dir.join(
                apk.file_name()
                    .expect(&msg_err_str("Couldn't get a filename.")),
            ),
        )?;

        patch_bootanim(bootanim_rc_path, booatnim_rc_gz_path, has_overlay)?;

        msg_sub("Finishing installation");

        if update {
            let data_adb = PathBuf::from(get_data_home()?).join("waydroid/data/adb");
            if data_adb.join("magisk").exists() {
                fs::remove_dir_all(data_adb.join("magisk"))?;
            }
            cp_dir(magisk_dir, data_adb.clone())?;
        }

        if !has_overlay {
            umount_system(false)?;
        }

        fs::remove_dir_all(tempdir)?;
        if !waydroid.get_session().is_empty() && !update {
            msg_sub("Starting Waydroid");
            if let Err(e) = waydroid.start() {
                msg_err(&format!("Couldn't start Waydroid container.\n{}", e));
                msg_end("Installation completed, start Waydroid manually");
            } else {
                msg_end("Installation completed");
            }
            msg_regular("Run \'sudo wsu setup\' after Waydroid starts or install the manager.");
        } else if !update {
            msg_end("Installation completed, start Waydroid manually");
            msg_regular("Run \'sudo wsu setup\' after Waydroid starts or install the manager.");
        }
    } else {
        msg_regular("KitsuneMagisk is already installed!");
    }
    Ok(())
}

pub fn update(arch: &str) -> anyhow::Result<()> {
    let mut waydroid = WaydroidContainer::new()?;
    if !waydroid.is_container_running()? {
        return Err(anyhow!("Waydroid container isn't running!"));
    }
    if !magisk_is_installed()? || !magisk_is_set_up()? {
        return Err(anyhow!("KitsuneMagisk is not installed."));
    }
    let tempdir = temp_dir().join("waydroidsu");
    if !tempdir.exists() {
        fs::create_dir(&tempdir)?;
    }
    let json_file = tempdir.join("canary.json");
    download_file(
        "https://raw.githubusercontent.com/mistrmochov/KitsuneMagisk-Waydroid/refs/heads/kitsune/canary.json",
        &json_file.to_string_lossy(),
        true,
    )?;
    let json_data = fs::read_to_string(json_file)?;
    let (version_online, _) = get_magisk_info(&json_data)?;
    let magisk = Magisk::new()?;
    if !magisk.version().contains(&version_online) {
        if !magisk.version().contains("Waydroid") {
            return Err(anyhow!(
                "Local version of Magisk not recognized, please reinstall!"
            ));
        }
        msg_main("Updating KitsuneMagisk...");
        remove(false, true)?;
        if let Err(e) = install(arch, "", true) {
            msg_err(&e.to_string());
            remove(true, false)?;
            msg_err("Update has failed, reinstall Magisk");
            return Ok(());
        }
        waydroid_su(vec!["chmod", "-R", "755", "/data/adb/magisk/"])?;
        waydroid_su(vec!["chown", "-R", "0:0", "/data/adb/magisk"])?;
        waydroid_su(vec!["cp", "/data/adb/magisk/magisk.apk", "/data/local/tmp"])?;
        waydroid_su(vec!["pm", "install", "/data/local/tmp/magisk.apk"])?;
        waydroid_su(vec!["rm", "/data/local/tmp/magisk.apk"])?;

        msg_end("Update completed, restarting Waydroid.");
        waydroid.restart_countdown()?;
    } else {
        msg_regular("KitsuneMagisk is up to date");
    }

    Ok(())
}

pub fn remove(recover: bool, update: bool) -> anyhow::Result<()> {
    if !magisk_is_installed()? && !recover {
        return Err(anyhow!("KitsuneMagisk is not installed!"));
    }
    if recover {
        msg_end(&"Aborting...".red());
    } else if !update {
        msg_main("Removing KitsuneMagisk...");
    }
    let mut waydroid = WaydroidContainer::new()?;
    if waydroid.is_session_running(true, true)? && !update {
        msg_sub("Stopping Waydroid");
        waydroid.stop(true)?;
    }
    let has_overlay = has_overlay().expect(&msg_err_str(
        "Couldn't reach the \"mount_overlays\" config.",
    ));
    let tempdir = temp_dir().join("waydroidsu");
    let rootfs;
    let overlay_rw = PathBuf::from(WAYDROID_DIR).join("overlay_rw/system");
    if has_overlay {
        rootfs = PathBuf::from(WAYDROID_DIR).join("overlay");
    } else {
        if !is_mounted_at(&tempdir.join("mnt").to_string_lossy())? {
            if let OtherOk(mount) = mount_system(waydroid.clone(), false, false) {
                if !mount {
                    return Err(anyhow!("Failed to mount system."));
                }
            } else {
                return Err(anyhow!("Failed to mount system."));
            }
        }
        rootfs = tempdir.join("mnt");
    }
    let magisk_dir = rootfs.join(MAGISK_DIR);
    let bootanim_rc_path = rootfs.join(BOOTANIM_RC_PATH);
    let booatnim_rc_gz_path = rootfs.join(BOOTANIM_RC_GZ_PATH);

    msg_sub("Removing files");
    if magisk_dir.exists() {
        fs::remove_dir_all(magisk_dir)?;
    }
    if rootfs.join("sbin").exists() {
        fs::remove_dir_all(rootfs.join("sbin"))?;
    }
    if rootfs.join("system/addon.d/99-magisk.sh").exists() {
        fs::remove_file(rootfs.join("system/addon.d/99-magisk.sh"))?;
    }

    restore_bootanim(bootanim_rc_path, booatnim_rc_gz_path, has_overlay)?;

    if overlay_rw.join(MAGISK_DIR).exists() {
        fs::remove_dir_all(overlay_rw.join(MAGISK_DIR))?;
    }
    if overlay_rw.join(BOOTANIM_RC_PATH).exists() {
        fs::remove_file(overlay_rw.join(BOOTANIM_RC_PATH))?;
    }
    if overlay_rw.join(BOOTANIM_RC_GZ_PATH).exists() {
        fs::remove_file(overlay_rw.join(BOOTANIM_RC_GZ_PATH))?;
    }
    if overlay_rw.join("system/addon.d/99-magisk.sh").exists() {
        fs::remove_file(overlay_rw.join("system/addon.d/99-magisk.sh"))?;
    }

    if !has_overlay {
        if let OtherOk(mount) = umount_system(false) {
            if !mount {
                return Err(anyhow!("Failed to umount system."));
            }
        } else {
            return Err(anyhow!("Failed to umount system."));
        }
    }

    if tempdir.exists() {
        fs::remove_dir_all(tempdir)?;
    }

    if !waydroid.get_session().is_empty() && !update {
        msg_sub("Starting Waydroid");
        if let Err(e) = waydroid.start() {
            msg_err(&format!("Couldn't start Waydroid container.\n{}", e));
            msg_end("Removal completed.");
            msg_regular("Start Waydroid manually.");
        } else {
            msg_end("Removal completed.");
        }
    } else if !update {
        msg_end("Removal completed.");
        msg_regular("Start Waydroid manually.");
    }
    Ok(())
}

pub fn setup() -> anyhow::Result<()> {
    let mut waydroid = WaydroidContainer::new()?;
    if !waydroid.is_container_running()? {
        return Err(anyhow!("Waydroid container isn't running!"));
    }

    if !magisk_is_installed()? {
        return Err(anyhow!("KitsuneMagisk ins't installed!"));
    }

    msg_main("Running additional setup...");
    waydroid_su(vec!["rm", "-rf", "/data/adb/magisk"])?;
    waydroid_su(vec!["mkdir", "-p", "/data/adb/magisk"])?;
    waydroid_su(vec!["chmod", "700", "/data/adb"])?;
    waydroid_su(vec!["cp", "-r", "/system/etc/init/magisk", "/data/adb/"])?;
    waydroid_su(vec!["chmod", "-R", "755", "/data/adb/magisk/"])?;
    waydroid_su(vec!["chown", "-R", "0:0", "/data/adb/magisk"])?;
    waydroid_su(vec![
        "cp",
        "/system/etc/init/magisk/magisk.apk",
        "/data/local/tmp",
    ])?;
    waydroid_su(vec!["pm", "install", "/data/local/tmp/magisk.apk"])?;
    waydroid_su(vec!["rm", "/data/local/tmp/magisk.apk"])?;

    waydroid.restart_countdown()?;
    Ok(())
}

fn patch_bootanim(
    bootanim_rc_path: PathBuf,
    booatnim_rc_gz_path: PathBuf,
    has_overlay: bool,
) -> anyhow::Result<()> {
    if booatnim_rc_gz_path.exists() {
        fs::remove_file(booatnim_rc_gz_path.clone())?;
    }
    if bootanim_rc_path.exists() {
        fs::remove_file(bootanim_rc_path.clone())?;
    }

    File::create(bootanim_rc_path.clone())?;
    if !has_overlay {
        msg_sub("Backing up bootanim.rc");
        fs::write(bootanim_rc_path.clone(), BOOTANIM_RC)?;
        gzip_compress(
            &bootanim_rc_path.to_string_lossy(),
            &booatnim_rc_gz_path.to_string_lossy(),
        )?;
    }

    msg_sub("Patching bootanim.rc");
    let x = generate_random_string(15);
    let y = generate_random_string(15);
    let mut bootanim_rc_magisk = BOOTANIM_RC_MAGISK.replace("magisk_service_x", &x);
    bootanim_rc_magisk = bootanim_rc_magisk.replace("magisk_service_y", &y);
    fs::write(bootanim_rc_path, bootanim_rc_magisk)?;
    Ok(())
}

fn restore_bootanim(
    bootanim_rc_path: PathBuf,
    booatnim_rc_gz_path: PathBuf,
    has_overlay: bool,
) -> anyhow::Result<()> {
    msg_sub("Restoring bootanim.rc");
    if !has_overlay {
        if booatnim_rc_gz_path.exists() {
            if bootanim_rc_path.exists() {
                fs::remove_file(bootanim_rc_path.clone())?;
            }
            gzip_decompress(
                &booatnim_rc_gz_path.to_string_lossy(),
                &bootanim_rc_path.to_string_lossy(),
            )?;
            fs::remove_file(booatnim_rc_gz_path)?;
        } else {
            if !bootanim_rc_path.exists() {
                File::create(bootanim_rc_path.clone())?;
            }
            fs::write(bootanim_rc_path, BOOTANIM_RC)?;
        }
    } else {
        if bootanim_rc_path.exists() {
            fs::remove_file(bootanim_rc_path)?;
        }
        if booatnim_rc_gz_path.exists() {
            fs::remove_file(booatnim_rc_gz_path)?;
        }
    }
    Ok(())
}
