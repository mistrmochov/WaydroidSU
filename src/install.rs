use crate::constants::*;
use crate::container::{WaydroidContainer, has_overlay};
use crate::magisk::Magisk;
use crate::magisk_files::{
    check_uninstall_manager, clean_up, magisk_is_installed, magisk_is_set_up, patch_bootanim,
    patch_init_zygote, restore_bootanim, restore_init_zygote, waydroid_su,
};
use crate::selinux::*;
use crate::utils::*;
use crate::{get_data_home, msg_end, msg_err, msg_err_str, msg_main, msg_regular, msg_sub};
use anyhow::{Ok, anyhow};
use colored::*;
use std::env::temp_dir;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::result::Result::Ok as OtherOk;

pub fn install(arch: &str, custom_apk: &str, update: bool, new: bool) -> anyhow::Result<()> {
    if !magisk_is_installed()? || update {
        let mut waydroid = WaydroidContainer::new()?;
        let has_overlay = has_overlay().expect(&msg_err_str(
            "Couldn't reach the \"mount_overlays\" config.",
        ));

        if !has_overlay {
            print!(
                "[{}] {} {} ",
                "WSU".blue().bold(),
                "Your setup has \"mount_overlays\" disabled, do you wish to modify system and vendor images?"
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
            msg_main("Installing Magisk...");
        }

        if waydroid.is_session_running(true, true)? && !(update || new) {
            msg_sub("Stopping Waydroid session");
            waydroid.stop(true)?;
        }

        if update || new {
            if !waydroid.is_container_running()? {
                return Err(anyhow!("Waydroid container isn't running!"));
            }
        }

        let tempdir = temp_dir().join("waydroidsu");
        let magisk_tmp = tempdir.join("magisk");
        create_tmpdir()?;

        let apk = resolve_apk(custom_apk, new.clone(), tempdir.clone())?;

        if !magisk_tmp.exists() {
            fs::create_dir(magisk_tmp.clone())?;
        }
        msg_sub("Extracting Magisk");
        unzip_file(&apk.to_string_lossy(), &magisk_tmp.to_string_lossy())?;
        let (libs, libs32, assets) = prepare_libs(magisk_tmp.clone(), arch, new.clone())?;

        let rootfs = if has_overlay {
            PathBuf::from(WAYDROID_DIR).join("overlay")
        } else {
            tempdir.join("mnt")
        };
        let overlay_rw = PathBuf::from(WAYDROID_DIR).join("overlay_rw/system");
        let magisk_dir = rootfs.join(MAGISK_DIR);
        let bootanim_rc_path = rootfs.join(BOOTANIM_RC_PATH);
        let bootanim_rc_gz_path = rootfs.join(BOOTANIM_RC_GZ_PATH);

        let waydroid_data = PathBuf::from(get_data_home()?).join("waydroid/data");

        if new {
            patch_sepolicy_prepare(waydroid_data.clone(), libs.join("libmagiskpolicy.so"))?;
            waydroid_su(
                vec!["cp", "/system/etc/init/hw/init.zygote*", "/data/local/tmp/"],
                true,
            )?;
            msg_sub("Stopping Waydroid session");
            waydroid.stop(true)?;
        }

        if !has_overlay && !is_mounted_at(&tempdir.join("mnt").to_string_lossy())? {
            mount_system(waydroid.clone(), false)?;
        }

        clean_up(rootfs.clone(), has_overlay, overlay_rw)?;

        create_dir_check(magisk_dir.clone(), true)?;
        create_dir_check(rootfs.join("system/addon.d"), has_overlay)?;
        create_dir_check(rootfs.join(MAGISKTMP), true)?;

        msg_sub("Copying scripts and binaries");
        for entry in fs::read_dir(libs)? {
            let path = entry?.path();

            let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if file_name.starts_with("lib") && file_name.ends_with(".so") {
                let target_name = file_name.trim_start_matches("lib").trim_end_matches(".so");
                fs::copy(&path, magisk_dir.join(target_name))?;
            }
        }

        let lib_name = if new {
            "libmagisk.so"
        } else {
            "libmagisk32.so"
        };
        fs::copy(libs32.join(lib_name), magisk_dir.join("magisk32"))?;

        let mut required_files = vec!["boot_patch.sh", "util_functions.sh", "addon.d.sh"];

        if new {
            required_files.extend_from_slice(&[
                "app_functions.sh",
                "module_installer.sh",
                "uninstaller.sh",
            ]);
        }

        for entry in fs::read_dir(&assets)? {
            let path = entry?.path();
            let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
                continue;
            };

            if required_files.contains(&file_name) {
                fs::copy(&path, magisk_dir.join(file_name))?;
            }
        }

        chmod_755_recursive(magisk_dir.clone())?;
        cp_dir(assets.join("chromeos"), magisk_dir.clone())?;
        chmod_755_recursive(magisk_dir.join("chromeos"))?;
        fs::copy(assets.join("stub.apk"), magisk_dir.join("stub.apk"))?;

        let apk_name = apk
            .file_name()
            .ok_or_else(|| anyhow!(msg_err_str("Couldn't get a filename.")))?;
        fs::copy(&apk, magisk_dir.join(apk_name))?;

        if new {
            patch_sepolicy(magisk_dir.clone(), rootfs.clone(), waydroid_data.clone())?;
            patch_init_zygote(rootfs.clone(), waydroid_data)?;
            create_dir_check(rootfs.join("system/etc/init"), false)?;
        }

        patch_bootanim(bootanim_rc_path, bootanim_rc_gz_path, has_overlay, new)?;

        msg_sub("Finishing installation");

        if update {
            let data_adb = PathBuf::from(get_data_home()?).join("waydroid/data/adb");
            let magisk_path = data_adb.join("magisk");
            if magisk_path.exists() {
                fs::remove_dir_all(&magisk_path)?;
            }
            cp_dir(magisk_dir, data_adb)?;
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
        msg_regular("Magisk is already installed!");
    }
    Ok(())
}

pub fn update(arch: &str) -> anyhow::Result<()> {
    let mut waydroid = WaydroidContainer::new()?;
    if !waydroid.is_container_running()? {
        return Err(anyhow!("Waydroid container isn't running!"));
    }
    if !magisk_is_installed()? || !magisk_is_set_up()? {
        return Err(anyhow!("Magisk is not installed."));
    }
    let tempdir = temp_dir().join("waydroidsu");
    create_tmpdir()?;

    let magisk = Magisk::new()?;
    let local_version = magisk.version();
    let new = !local_version.contains("v27.2");

    let json_file = tempdir.join("canary.json");
    download_file(
        if new {
            "https://raw.githubusercontent.com/mistrmochov/MagiskForWaydroid/refs/heads/master/stable.json"
        } else {
            "https://raw.githubusercontent.com/mistrmochov/KitsuneMagisk-Waydroid/refs/heads/kitsune/canary.json"
        },
        &json_file.to_string_lossy(),
        true,
    )?;
    let json_data = fs::read_to_string(json_file)?;
    let (version_online, _) = get_magisk_info(&json_data)?;
    if !local_version.contains(&version_online) {
        if !local_version.contains("Waydroid") {
            return Err(anyhow!(
                "Local version of Magisk not recognized, please reinstall!"
            ));
        }
        msg_main("Updating Magisk...");
        remove(false, true)?;
        if let Err(e) = install(arch, "", true, false) {
            msg_err(&e.to_string());
            remove(true, false)?;
            msg_err("Update has failed, reinstall Magisk");
            return Ok(());
        }
        waydroid_su(vec!["chmod", "-R", "755", "/data/adb/magisk/"], false)?;
        waydroid_su(vec!["chown", "-R", "0:0", "/data/adb/magisk"], false)?;
        waydroid_su(
            vec!["cp", "/data/adb/magisk/magisk.apk", "/data/local/tmp"],
            false,
        )?;
        check_uninstall_manager()?;
        waydroid_su(vec!["pm", "install", "/data/local/tmp/magisk.apk"], false)?;
        waydroid_su(vec!["rm", "/data/local/tmp/magisk.apk"], false)?;

        msg_end("Update completed, restarting Waydroid.");
        waydroid.restart_countdown()?;
    } else {
        msg_regular("Magisk is up to date");
    }

    Ok(())
}

pub fn remove(recover: bool, update: bool) -> anyhow::Result<()> {
    if !magisk_is_installed()? && !recover {
        return Err(anyhow!("Magisk is not installed!"));
    }
    if recover {
        msg_end(&"Aborting...".red());
    } else if !update {
        msg_main(&format!("Removing Magisk..."));
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
    create_tmpdir()?;

    let overlay_rw = PathBuf::from(WAYDROID_DIR).join("overlay_rw/system");
    let rootfs = if has_overlay {
        PathBuf::from(WAYDROID_DIR).join("overlay")
    } else {
        if !is_mounted_at(&tempdir.join("mnt").to_string_lossy())? {
            if let OtherOk(mount) = mount_system(waydroid.clone(), false) {
                if !mount {
                    return Err(anyhow!("Failed to mount system."));
                }
            } else {
                return Err(anyhow!("Failed to mount system."));
            }
        }
        tempdir.join("mnt")
    };
    let bootanim_rc_path = rootfs.join(BOOTANIM_RC_PATH);
    let bootanim_rc_gz_path = rootfs.join(BOOTANIM_RC_GZ_PATH);

    msg_sub("Removing files");
    clean_up(rootfs.clone(), has_overlay, overlay_rw)?;
    restore_sepolicy(rootfs.clone(), has_overlay)?;
    restore_init_zygote(rootfs, has_overlay)?;
    restore_bootanim(bootanim_rc_path, bootanim_rc_gz_path, has_overlay)?;

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
        return Err(anyhow!("Magisk ins't installed!"));
    }

    msg_main("Running additional setup...");
    waydroid_su(vec!["rm", "-rf", "/data/adb/magisk"], false)?;
    waydroid_su(vec!["mkdir", "-p", "/data/adb/magisk"], false)?;
    waydroid_su(vec!["chmod", "700", "/data/adb"], false)?;
    waydroid_su(
        vec!["cp", "-r", "/system/etc/init/magisk/*", "/data/adb/magisk"],
        false,
    )?;
    waydroid_su(vec!["chmod", "-R", "755", "/data/adb/magisk/"], false)?;
    waydroid_su(vec!["chown", "-R", "0:0", "/data/adb/magisk"], false)?;
    waydroid_su(
        vec![
            "cp",
            "/system/etc/init/magisk/magisk.apk",
            "/data/local/tmp",
        ],
        false,
    )?;
    check_uninstall_manager()?;
    waydroid_su(vec!["pm", "install", "/data/local/tmp/magisk.apk"], false)?;
    waydroid_su(vec!["rm", "/data/local/tmp/magisk.apk"], false)?;

    waydroid.restart_countdown()?;
    Ok(())
}

fn resolve_apk(custom_apk: &str, new: bool, tempdir: PathBuf) -> anyhow::Result<PathBuf> {
    let apk = tempdir.join("magisk.apk");
    if custom_apk.is_empty() {
        let json_file = tempdir.join("channel.json");
        download_file(
            if new {
                "https://raw.githubusercontent.com/mistrmochov/MagiskForWaydroid/refs/heads/master/stable.json"
            } else {
                "https://raw.githubusercontent.com/mistrmochov/KitsuneMagisk-Waydroid/refs/heads/kitsune/canary.json"
            },
            &json_file.to_string_lossy(),
            true,
        )?;
        let json_data = fs::read_to_string(json_file)?;
        let (version, link) = get_magisk_info(&json_data)?;
        msg_sub(&format!("Downloading Magisk: {}", version.blue().bold()));
        download_file(&link, &apk.to_string_lossy(), false)?;
    } else {
        let path = PathBuf::from(custom_apk);
        if !path.exists() || !path.is_file() {
            return Err(anyhow!(
                "Custom APK file doesn't exist or isn't a valid file."
            ));
        }
        fs::copy(&path, &apk)?;
    }
    Ok(apk)
}

fn prepare_libs(
    magisk_tmp: PathBuf,
    arch: &str,
    new: bool,
) -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
    let libs = magisk_tmp.join("lib").join(arch);
    let libs32 = if arch == "x86_64" {
        magisk_tmp.join("lib/x86")
    } else {
        magisk_tmp.join("lib/armeabi-v7a")
    };
    let assets = magisk_tmp.join("assets");

    if !libs.exists() || !libs32.exists() || !assets.exists() {
        return Err(anyhow!("Structure of apk file hasn't been recognized."));
    }
    if libs.join("libmagisk.so").exists() && !new {
        return Err(anyhow!(
            "Structure of this apk file requires flag \'--new\'/\'-n\'"
        ));
    } else if libs.join("libmagisk64.so").exists() && new {
        return Err(anyhow!(
            "Structure of this apk file can't be installed with flag \'--new\'/\'-n\'"
        ));
    }

    Ok((libs, libs32, assets))
}
