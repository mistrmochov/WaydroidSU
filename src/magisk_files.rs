use crate::constants::*;
use crate::container::{WaydroidContainer, has_overlay};
use crate::magisk::Magisk;
use crate::print::{msg_err_str, msg_sub};
use crate::selinux::*;
use crate::utils::*;
use anyhow::{Ok, anyhow};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn magisk_is_installed() -> anyhow::Result<bool> {
    let mut waydroid = WaydroidContainer::new()?;
    let magisk_dir;
    if waydroid.is_session_running(true, true)? {
        magisk_dir = PathBuf::from(WAYDROID_DIR).join("rootfs/system/etc/init/magisk");
    } else if has_overlay().expect(&msg_err_str("Failed to get overlay result!")) {
        magisk_dir = PathBuf::from(WAYDROID_DIR).join("overlay/system/etc/init/magisk");
    } else {
        mount_system(waydroid, true)?;
        magisk_dir = PathBuf::from("/mnt/waydroid").join("system/etc/init/magisk");
        let magisk_dir_result = magisk_dir.exists();
        umount_system(true)?;
        return Ok(magisk_dir_result);
    }
    Ok(magisk_dir.exists())
}

pub fn magisk_is_set_up() -> anyhow::Result<bool> {
    let mut waydroid = WaydroidContainer::new()?;
    let magisk_init =
        if has_overlay().expect(&msg_err_str("Failed to get \"mount_overlays\" config,")) {
            PathBuf::from(WAYDROID_DIR)
                .join("overlay")
                .join(MAGISK_DIR)
                .join("magisk")
        } else {
            if !waydroid.is_session_running(true, true)? {
                return Err(anyhow!("Waydroid session isn't running!"));
            }
            PathBuf::from(WAYDROID_DIR)
                .join("rootfs")
                .join(MAGISK_DIR)
                .join("magisk")
        };
    let magisk_data = PathBuf::from(get_data_home()?).join("waydroid/data/adb/magisk/magisk");
    let magisk_init64 =
        if has_overlay().expect(&msg_err_str("Failed to get \"mount_overlays\" config,")) {
            PathBuf::from(WAYDROID_DIR)
                .join("overlay")
                .join(MAGISK_DIR)
                .join("magisk64")
        } else {
            if !waydroid.is_session_running(true, true)? {
                return Err(anyhow!("Waydroid session isn't running!"));
            }
            PathBuf::from(WAYDROID_DIR)
                .join("rootfs")
                .join(MAGISK_DIR)
                .join("magisk64")
        };
    let magisk_data64 = PathBuf::from(get_data_home()?).join("waydroid/data/adb/magisk/magisk64");
    Ok((magisk_init.exists() || magisk_init64.exists())
        && (magisk_data.exists() || magisk_data64.exists()))
}

pub fn get_status() -> anyhow::Result<(bool, String, bool)> {
    let mut waydroid = WaydroidContainer::new()?;
    if !waydroid.is_container_running()? {
        return Err(anyhow!("Waydroid container isn't running!"));
    }
    let installed = magisk_is_installed()? && magisk_is_set_up()?;

    let mut args = Vec::new();
    args.push("pidof");
    args.push("magiskd");
    let daemon_running = if installed {
        if let Err(_) = waydroid_su(args, false) {
            false
        } else {
            true
        }
    } else {
        false
    };
    let (zygisk, version) = if daemon_running {
        let mut magisk = Magisk::new()?;
        (magisk.get_zygisk()?, magisk.version().to_string())
    } else {
        (false, String::new())
    };
    Ok((daemon_running, version, zygisk))
}

pub fn waydroid_su(args: Vec<&str>, force_no_su: bool) -> anyhow::Result<String> {
    let mut waydroid = WaydroidContainer::new()?;
    let selinux_enforcing = getenforce()?;
    if !waydroid.is_container_running()? {
        return Err(anyhow!("Waydroid container isn't running!"));
    }

    if !magisk_is_installed()? && !force_no_su {
        return Err(anyhow!("Magisk is not installed!"));
    }

    if args.is_empty() {
        return Err(anyhow!("su arguments are empty"));
    }
    let lxc = PathBuf::from(WAYDROID_DIR).join("lxc");
    let path_var = env::var("PATH")?;

    let args_string = args.join(" ");
    let full_command = if force_no_su {
        args_string
    } else {
        format!("/{}/su -c {}", MAGISKTMP, args_string)
    };

    let mut cmd = Command::new("lxc-attach");
    cmd.args(["-P", lxc.to_string_lossy().trim(), "-n", "waydroid", "--"]);

    if selinux_enforcing {
        cmd.args(["sh", "-c", &full_command]);
    } else {
        if force_no_su {
            cmd.args(["sh", "-c", &full_command]);
        } else {
            cmd.args([&format!("/{}/su", MAGISKTMP), "-c"]).args(args);
        }
    }

    cmd.env(
        "PATH",
        format!("{path_var}:/system/bin:/vendor/bin:/product/bin"),
    );

    let out = cmd.output()?;

    if !out.status.success() {
        let error = if out.stderr.is_empty() {
            String::from_utf8_lossy(&out.stdout)
        } else {
            String::from_utf8_lossy(&out.stderr)
        };
        return Err(anyhow!(error.trim().to_string()));
    }
    let out_send = if out.stdout.is_empty() {
        String::from_utf8_lossy(&out.stderr)
    } else {
        String::from_utf8_lossy(&out.stdout)
    };
    Ok(out_send.trim().to_string())
}

fn move_from_overlay_rw(overlay_rw: PathBuf, overlay: PathBuf) -> anyhow::Result<()> {
    if overlay_rw.exists() {
        remove_check(overlay.clone())?;
        fs::copy(overlay_rw.clone(), overlay)?;
        remove_check(overlay_rw)?;
    }
    Ok(())
}

pub fn clean_up(
    rootfs: PathBuf,
    has_overlay: bool,
    overlay_rw: PathBuf,
    waydroid_data: PathBuf,
) -> anyhow::Result<()> {
    let rm_adb = !waydroid_data.to_string_lossy().is_empty();
    let adb_magisk = waydroid_data.join("adb/magisk");

    let common_paths = [
        overlay_rw.join(MAGISK_DIR),
        rootfs.join(MAGISK_DIR),
        overlay_rw.join("system/addon.d/99-magisk.sh"),
        rootfs.join("system/addon.d/99-magisk.sh"),
    ];

    for path in common_paths {
        remove_check(path)?;
    }

    if rm_adb {
        remove_check(adb_magisk)?;
    }

    if has_overlay {
        let vendor_selinux = rootfs.join("vendor/etc/selinux");
        let vendor_selinux_rw = PathBuf::from(WAYDROID_DIR).join("overlay_rw/vendor/etc/selinux");

        let move_pairs = vec![
            (
                vendor_selinux_rw.join("vendor_file_contexts"),
                vendor_selinux.join("vendor_file_contexts"),
            ),
            (
                vendor_selinux_rw.join("vendor_file_contexts.gz"),
                vendor_selinux.join("vendor_file_contexts.gz"),
            ),
            (
                overlay_rw.join("system/etc/init/hw/init.zygote32.rc"),
                rootfs.join("system/etc/init/hw/init.zygote32.rc"),
            ),
            (
                overlay_rw.join("system/etc/init/hw/init.zygote64_32.rc"),
                rootfs.join("system/etc/init/hw/init.zygote64_32.rc"),
            ),
            (
                overlay_rw.join("system/etc/init/hw/init.zygote32.rc.gz"),
                rootfs.join("system/etc/init/hw/init.zygote32.rc.gz"),
            ),
            (
                overlay_rw.join("system/etc/init/hw/init.zygote64_32.rc.gz"),
                rootfs.join("system/etc/init/hw/init.zygote64_32.rc.gz"),
            ),
            (
                overlay_rw.join(BOOTANIM_RC_PATH),
                rootfs.join(BOOTANIM_RC_PATH),
            ),
            (
                overlay_rw.join(BOOTANIM_RC_GZ_PATH),
                rootfs.join(BOOTANIM_RC_GZ_PATH),
            ),
        ];

        let removals = [
            vendor_selinux.join("precompiled_sepolicy"),
            vendor_selinux_rw.join("precompiled_sepolicy"),
            vendor_selinux.join("precompiled_sepolicy.gz"),
            vendor_selinux_rw.join("precompiled_sepolicy.gz"),
            rootfs.join(MAGISKTMP),
        ];

        for path in removals {
            remove_check(path)?;
        }

        for (src, dest) in move_pairs {
            move_from_overlay_rw(src, dest)?;
        }
    }
    Ok(())
}

fn backup_bootanim(bootanim_rc_path: PathBuf, bootanim_rc_gz_path: PathBuf) -> anyhow::Result<()> {
    msg_sub("Backing up bootanim.rc");
    gzip_compress(
        &bootanim_rc_path.to_string_lossy(),
        &bootanim_rc_gz_path.to_string_lossy(),
    )?;
    Ok(())
}

pub fn patch_bootanim(
    bootanim_rc_path: PathBuf,
    bootanim_rc_gz_path: PathBuf,
    has_overlay: bool,
    kitsune: bool,
) -> anyhow::Result<()> {
    if bootanim_rc_path.exists() {
        if !bootanim_rc_gz_path.exists() {
            backup_bootanim(bootanim_rc_path.clone(), bootanim_rc_gz_path.clone())?;
        }
    } else if !bootanim_rc_gz_path.exists() && !has_overlay {
        File::create(bootanim_rc_path.clone())?;
        fs::write(bootanim_rc_path.clone(), BOOTANIM_RC)?;
        backup_bootanim(bootanim_rc_path.clone(), bootanim_rc_gz_path.clone())?;
    } else {
        File::create(bootanim_rc_path.clone())?;
    }

    msg_sub("Patching bootanim.rc");
    let x = generate_random_string(15);
    let y = generate_random_string(15);
    let mut bootanim_rc_magisk = if !kitsune {
        BOOTANIM_RC_MAGISK_NEW.replace("magisk_service_x", &x)
    } else {
        BOOTANIM_RC_MAGISK.replace("magisk_service_x", &x)
    };
    bootanim_rc_magisk = bootanim_rc_magisk.replace("magisk_service_y", &y);
    fs::write(bootanim_rc_path, bootanim_rc_magisk)?;
    Ok(())
}

pub fn restore_bootanim(
    bootanim_rc_path: PathBuf,
    bootanim_rc_gz_path: PathBuf,
    has_overlay: bool,
) -> anyhow::Result<()> {
    if bootanim_rc_gz_path.exists() {
        remove_check(bootanim_rc_path.clone())?;
        msg_sub("Restoring bootanim.rc");
        gzip_decompress(
            &bootanim_rc_gz_path.to_string_lossy(),
            &bootanim_rc_path.to_string_lossy(),
        )?;
        fs::remove_file(bootanim_rc_gz_path)?;
    } else if bootanim_rc_path.exists() {
        if has_overlay {
            fs::remove_file(bootanim_rc_path.clone())?;
        } else {
            msg_sub("Restoring bootanim.rc");
            fs::write(bootanim_rc_path.clone(), BOOTANIM_RC)?;
        }
    }
    Ok(())
}

fn inject_zygote_restart<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let input_string = fs::read_to_string(&path)?;
    if !input_string
        .contains("    exec u:r:magisk:s0 0 0 -- /debug_ramdisk/magisk --zygote-restart")
    {
        let input = fs::File::open(&path)?;
        let reader = BufReader::new(input);
        let mut output = Vec::new();
        for line in reader.lines() {
            let line = line?;
            output.push(line.clone());

            if line.contains("service zygote ") {
                output.push(
                    "    exec u:r:magisk:s0 0 0 -- /debug_ramdisk/magisk --zygote-restart"
                        .to_string(),
                );
            }
        }

        fs::write(path, output.join("\n"))?;
    }

    Ok(())
}

pub fn restore_init_zygote(rootfs: PathBuf, has_overlay: bool) -> anyhow::Result<()> {
    let mut msg = false;
    let zygotes = [
        ("init.zygote32.rc", "init.zygote32.rc.gz"),
        ("init.zygote64_32.rc", "init.zygote64_32.rc.gz"),
    ];

    for (plain, gz) in zygotes {
        let plain_path = rootfs.join(format!("system/etc/init/hw/{}", plain));
        let gz_path = rootfs.join(format!("system/etc/init/hw/{}", gz));

        if has_overlay || gz_path.exists() {
            if remove_check(plain_path.clone())? {
                msg = true;
            }
        }

        if gz_path.exists() {
            msg = true;
            gzip_decompress(&gz_path.to_string_lossy(), &plain_path.to_string_lossy())?;
            remove_check(gz_path)?;
        }
    }

    if msg {
        msg_sub("Restoring init zygote");
    }

    Ok(())
}

pub fn patch_init_zygote(rootfs: PathBuf, waydroid_data: PathBuf) -> anyhow::Result<()> {
    create_dir_check(rootfs.clone().join("system/etc/init/hw"), false)?;

    let zygotes = [
        ("init.zygote32.rc", "init.zygote32.rc.gz"),
        ("init.zygote64_32.rc", "init.zygote64_32.rc.gz"),
    ];

    msg_sub("Injecting zygote restart");
    for (plain, gz) in zygotes {
        let plain_path = rootfs.join(format!("system/etc/init/hw/{}", plain));
        let plain_path_data = waydroid_data.join("local/tmp").join(plain);
        let gz_path = rootfs.join(format!("system/etc/init/hw/{}", gz));
        if !plain_path.exists() {
            fs::copy(&plain_path_data, plain_path.clone())?;
        } else if !gz_path.exists() {
            msg_sub(&format!("Backing up \'{}\'", plain));
            gzip_compress(&plain_path.to_string_lossy(), &gz_path.to_string_lossy())?;
        }

        set_selinux_attr(
            &plain_path.to_string_lossy(),
            "security.selinux",
            "u:object_r:system_file:s0",
        )?;
        inject_zygote_restart(&plain_path)?;

        fs::remove_file(plain_path_data)?;
    }
    Ok(())
}

pub fn check_uninstall_manager() -> anyhow::Result<()> {
    let packages = waydroid_su(vec!["pm", "list", "packages"], false)?;
    if packages.contains("com.topjohnwu.magisk") {
        waydroid_su(vec!["pm", "uninstall", "com.topjohnwu.magisk"], false)?;
    }
    if packages.contains("io.github.huskydg.magisk") {
        waydroid_su(vec!["pm", "uninstall", "io.github.huskydg.magisk;"], false)?;
    }
    Ok(())
}
