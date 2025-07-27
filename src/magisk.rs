use crate::container::WaydroidContainer;
use crate::magisk_files::{magisk_is_installed, magisk_is_set_up, waydroid_su};
use crate::selinux::getenforce;
use crate::utils::{create_tmpdir, unzip_file};
use crate::{get_data_home, msg_end, msg_err, msg_err_str, msg_main, msg_regular, msg_sub};
use anyhow::{Ok, anyhow};
use colored::*;
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::result::Result::Ok as OtherOk;

pub struct Magisk {
    waydroid: WaydroidContainer,
    installed: bool,
    version: String,
    modules_path: PathBuf,
}

impl Magisk {
    pub fn new() -> anyhow::Result<Self> {
        let waydroid = WaydroidContainer::new()?;
        let installed = magisk_is_installed()? && magisk_is_set_up()?;
        let version_full = if installed {
            waydroid_su(vec!["magisk", "-v"], false)?
        } else {
            return Err(anyhow!("Magisk isn't installed!"));
        };

        let version = if version_full.trim().contains(":MAGISK:R") {
            version_full
                .trim()
                .trim_end_matches(":MAGISK:R")
                .to_string()
        } else {
            "".to_string()
        };

        Ok(Self {
            waydroid,
            installed,
            version,
            modules_path: PathBuf::from(get_data_home()?).join("waydroid/data/adb/modules"),
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn list_modules(&mut self) -> anyhow::Result<()> {
        if !self.waydroid.is_session_running(true, true)? {
            return Err(anyhow!("Waydroid session isn't running!"));
        }

        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }

        let path = &self.modules_path;
        if !path.exists() {
            msg_regular("No modules installed.");
            return Ok(());
        }

        let entries = match fs::read_dir(path) {
            OtherOk(entries) => entries.collect::<Result<Vec<_>, _>>()?,
            Err(_) => {
                msg_regular("No modules installed.");
                return Ok(());
            }
        };

        if entries.is_empty() {
            msg_regular("No modules installed.");
            return Ok(());
        }

        let mut modules = Vec::new();

        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Failed to get module name"))?;
            modules.push(name);
        }

        msg_regular(&format!("Modules: {}", modules.len().to_string().blue()));

        for module in modules {
            let module_path = path.join(&module);
            let disabled = module_path.join("disable").exists();
            if disabled {
                msg_sub(&format!("{module} (disabled)"));
            } else {
                msg_sub(&module);
            }
        }

        Ok(())
    }

    fn is_module_disabled(&mut self, module: &str) -> anyhow::Result<bool> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }

        let module_dir = self.modules_path.join(module);
        if !module_dir.exists() || !module_dir.is_dir() {
            return Err(anyhow!(format!("\'{}\' not found", module)));
        }

        Ok(module_dir.join("disable").exists() && module_dir.join("disable").is_file())
    }

    pub fn disable_module(&mut self, module: &str) -> anyhow::Result<()> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }

        let module_dir = self.modules_path.join(module);
        if !module_dir.exists() || !module_dir.is_dir() {
            return Err(anyhow!(format!("\'{}\' not found", module)));
        }

        if !self.is_module_disabled(module)? {
            File::create(module_dir.join("disable"))?;
            msg_regular(&format!("Disabled: \'{}\'", module));
        } else {
            return Err(anyhow!(format!("\'{}\' already disabled", module)));
        }
        Ok(())
    }

    pub fn enable_module(&mut self, module: &str) -> anyhow::Result<()> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }

        let module_dir = self.modules_path.join(module);
        if !module_dir.exists() || !module_dir.is_dir() {
            return Err(anyhow!(format!("\'{}\' not found", module)));
        }

        if self.is_module_disabled(module)? {
            fs::remove_file(module_dir.join("disable"))?;
            msg_regular(&format!("Enabled: \'{}\'", module));
        } else {
            return Err(anyhow!(format!("\'{}\' already enabled", module)));
        }
        Ok(())
    }

    pub fn remove_module(&mut self, module: &str) -> anyhow::Result<()> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }

        let module_dir = self.modules_path.join(module);
        if !module_dir.exists() || !module_dir.is_dir() {
            return Err(anyhow!(format!("\'{}\' not found", module)));
        }
        fs::remove_dir_all(module_dir)?;
        msg_regular(&format!("Removed: \'{}\'", module));
        msg_regular("Reboot Waydroid to apply changes.");
        Ok(())
    }

    fn get_module_name(&self, path: &str) -> anyhow::Result<String> {
        let tmp = temp_dir().join("waydroidsu");
        create_tmpdir()?;
        let tmpdir = tmp.join("module_name");
        if !tmpdir.exists() {
            fs::create_dir_all(tmpdir.clone())?;
        }
        unzip_file(path, &tmpdir.to_string_lossy())?;

        let prop = tmpdir.join("module.prop");

        let file = fs::File::open(prop)?;
        let reader = BufReader::new(file);
        let mut props = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                props.insert(key.trim().to_string(), value.trim().to_string());
                if key == "id" {
                    if tmp.exists() {
                        fs::remove_dir_all(tmp)?;
                    }
                    return Ok(value.to_string());
                }
            }
        }

        if tmp.exists() {
            fs::remove_dir_all(tmp)?;
        }
        Err(anyhow!("Couldn't get name of the module!"))
    }

    pub fn install_module(&mut self, module: &str) -> anyhow::Result<()> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }
        let module_path = PathBuf::from(module);
        let tmp_dir = PathBuf::from(get_data_home()?).join("waydroid/data/local/tmp");

        if !module_path.exists() {
            return Err(anyhow!("No such file or directory."));
        }
        let filename = module_path
            .file_name()
            .expect(&msg_err_str("Failed to get file name."))
            .to_string_lossy();
        if !module_path
            .to_string_lossy()
            .trim()
            .to_ascii_lowercase()
            .ends_with(".zip")
        {
            return Err(anyhow!(format!("{} isn't a zip file", filename)));
        }

        let waydroid_module_path = PathBuf::from("/data/local/tmp").join(filename.to_string());
        let waydroid_module_path_string = waydroid_module_path.to_string_lossy().to_string();
        msg_main(&format!("Installing magisk module"));
        msg_sub("Copying to temporary directory");
        fs::copy(module_path.clone(), tmp_dir.join(filename.to_string()))?;
        msg_sub("Installing");
        if let Err(e) = waydroid_su(
            vec!["magisk", "--install-module", &waydroid_module_path_string],
            false,
        ) {
            return Err(anyhow!("Installation failed! - {}", e));
        }
        waydroid_su(vec!["rm", &waydroid_module_path.to_string_lossy()], false)?;
        let name = match self.get_module_name(&module_path.to_string_lossy()) {
            OtherOk(n) => n,
            Err(e) => {
                msg_err(&e.to_string());
                msg_end("Installation completed.");
                msg_regular("Reboot Waydroid to apply changes.");
                return Ok(());
            }
        };
        msg_end(&format!("module: \'{}\' has been installed", name));
        msg_regular("Reboot Waydroid to apply changes.");

        Ok(())
    }

    pub fn cmd(
        &mut self,
        applet: &str,
        args: Vec<&str>,
        force_no_su: bool,
    ) -> anyhow::Result<String> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }
        let mut args_new: Vec<&str> = Vec::with_capacity(args.len() + 1);
        args_new.push("magisk");
        args_new.push(&applet);
        args_new.extend(args.iter().map(|s| s));
        let out = waydroid_su(args_new, force_no_su)?;
        if applet != "--sqlite" {
            if !out.is_empty() {
                println!("{}", out.bold());
            }
        }
        Ok(out)
    }

    pub fn sqlite(&mut self, arg: &str, force_no_su: bool) -> anyhow::Result<String> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }
        Ok(self.cmd("--sqlite", vec![arg], force_no_su)?)
    }

    pub fn get_zygisk(&mut self) -> anyhow::Result<bool> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }
        let getenforce = getenforce()?;
        let zygisk_str = self.sqlite(
            "\"SELECT value FROM settings WHERE key == 'zygisk'\"",
            getenforce,
        )?;

        if let Some(zygisk) = zygisk_str.trim().split('=').last() {
            return Ok(zygisk == "1");
        }
        Err(anyhow!("Couldn't get the value of zygisk!"))
    }

    pub fn set_zygisk(&mut self, enabled: bool) -> anyhow::Result<()> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("Magisk isn't installed!"));
        }
        let getenforce = getenforce()?;
        if enabled {
            self.sqlite(
                "\"REPLACE INTO settings (key,value) VALUES('zygisk',1)\"",
                getenforce,
            )?;
        } else {
            self.sqlite(
                "\"REPLACE INTO settings (key,value) VALUES('zygisk',0)\"",
                getenforce,
            )?;
        }
        Ok(())
    }
}
