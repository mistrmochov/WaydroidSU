use crate::constants::*;
use crate::magisk_files::waydroid_su;
use crate::msg_sub;
use crate::utils::*;
use anyhow::Ok;
use libc::setxattr;
use std::ffi::CString;
use std::fs::{self, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::result::Result::Ok as OtherOk;

pub fn getenforce() -> anyhow::Result<bool> {
    let output;
    if let OtherOk(out) = Command::new("getenforce").output() {
        output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    } else {
        output = String::new();
    }

    Ok(output == "Enforcing")
}

pub fn set_selinux_attr(file: &str, name: &str, value: &str) -> anyhow::Result<()> {
    let path = CString::new(file)?;
    let name = CString::new(name)?;
    let value = CString::new(value)?;

    let ret = unsafe {
        setxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr() as *const _,
            value.as_bytes().len(),
            0,
        )
    };

    if ret == 0 {
        Ok(())
    } else {
        // setattr failed, ignoring
        Ok(())
    }
}

pub fn set_selinux_attr_recursive(dir: PathBuf, name: &str, value: &str) -> anyhow::Result<()> {
    if let OtherOk(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let OtherOk(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    set_selinux_attr_recursive(path, name, value)?;
                } else {
                    set_selinux_attr(&path.to_string_lossy(), name, value)?;
                }
            }
        }
    }
    Ok(())
}

pub fn restore_sepolicy(rootfs: PathBuf, has_overlay: bool) -> anyhow::Result<()> {
    let waydroid_vendor_selinux = rootfs.clone().join("vendor/etc/selinux");
    let se_files = [
        ("precompiled_sepolicy", "precompiled_sepolicy.gz"),
        ("vendor_file_contexts", "vendor_file_contexts.gz"),
    ];
    let mut msg = false;

    for (plain, gz) in se_files {
        let plain_path = waydroid_vendor_selinux.join(plain);
        let gz_path = waydroid_vendor_selinux.join(gz);

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
        msg_sub("Restoring sepolicy");
    }
    Ok(())
}

pub fn patch_sepolicy_prepare(waydroid_data: PathBuf, magiskpolicy: PathBuf) -> anyhow::Result<()> {
    fs::copy(magiskpolicy, waydroid_data.join("local/tmp/magiskpolicy"))?;
    waydroid_su(vec!["chmod", "755", "/data/local/tmp/magiskpolicy"], true)?;
    waydroid_su(
        vec![
            "/data/local/tmp/magiskpolicy",
            "--load",
            "/vendor/etc/selinux/precompiled_sepolicy",
            "--save",
            "/data/local/tmp/precompiled_sepolicy",
        ],
        true,
    )?;
    waydroid_su(
        vec![
            "cp",
            "/vendor/etc/selinux/vendor_file_contexts",
            "/data/local/tmp/",
        ],
        true,
    )?;
    waydroid_su(vec!["rm", "/data/local/tmp/magiskpolicy"], true)?;
    Ok(())
}

pub fn patch_sepolicy(
    magisk_dir: PathBuf,
    rootfs: PathBuf,
    waydroid_data: PathBuf,
) -> anyhow::Result<()> {
    let vendor_selinux = rootfs.join("vendor/etc/selinux");
    let init_hw_dir = rootfs.join("system/etc/init/hw");

    let precompiled = vendor_selinux.join("precompiled_sepolicy");
    let precompiled_gz = vendor_selinux.join("precompiled_sepolicy.gz");
    let contexts_file = vendor_selinux.join("vendor_file_contexts");
    let contexts_file_gz = vendor_selinux.join("vendor_file_contexts.gz");

    msg_sub("Patching sepolicy");

    let loadpolicy_path = magisk_dir.join("loadpolicy.sh");
    remove_check(loadpolicy_path.clone())?;
    fs::write(&loadpolicy_path, LOADPOLICY)?;
    fs::set_permissions(&loadpolicy_path, Permissions::from_mode(0o755))?;

    set_selinux_attr_recursive(magisk_dir, "security.selinux", "u:object_r:magisk_file:s0")?;

    create_dir_check(vendor_selinux, false)?;
    create_dir_check(init_hw_dir, false)?;

    if precompiled.exists() && !precompiled_gz.exists() {
        msg_sub("Backing up 'precompiled_sepolicy'");
        gzip_compress(
            &precompiled.to_string_lossy(),
            &precompiled_gz.to_string_lossy(),
        )?;
    }
    remove_check(precompiled.clone())?;
    fs::copy(
        waydroid_data.join("local/tmp/precompiled_sepolicy"),
        &precompiled,
    )?;

    if !contexts_file.exists() {
        fs::copy(
            waydroid_data.join("local/tmp/vendor_file_contexts"),
            &contexts_file,
        )?;
    } else if !contexts_file_gz.exists() {
        msg_sub("Backing up 'vendor_file_contexts'");
        gzip_compress(
            &contexts_file.to_string_lossy(),
            &contexts_file_gz.to_string_lossy(),
        )?;
    }

    fs::remove_file(waydroid_data.join("local/tmp/precompiled_sepolicy"))?;
    fs::remove_file(waydroid_data.join("local/tmp/vendor_file_contexts"))?;

    let file_string = fs::read_to_string(&contexts_file)?;
    let mut file = OpenOptions::new().append(true).open(&contexts_file)?;
    let contexts = [
        "/debug_ramdisk(/.*)?    u:object_r:magisk_file:s0",
        "/data/adb/magisk(/.*)?   u:object_r:magisk_file:s0",
    ];
    for line in contexts {
        if !file_string.contains(line) {
            writeln!(file, "{line}")?;
        }
    }

    Ok(())
}
