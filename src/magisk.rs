use crate::container::WaydroidContainer;
use crate::magisk_files::{magisk_is_installed, magisk_is_set_up, waydroid_su};
use crate::utils::{create_tmpdir, getenforce, unzip_file};
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
    getenforce: bool,
}

impl Magisk {
    pub fn new() -> anyhow::Result<Self> {
        let waydroid = WaydroidContainer::new()?;
        let installed = magisk_is_installed()? && magisk_is_set_up()?;
        let getenforce = getenforce()?;
        let (_, version_full) = if installed {
            waydroid_su(if getenforce {
                vec!["sh", "-c", "magisk -v"]
            } else {
                vec!["magisk", "-v"]
            })?
        } else {
            (true, String::new())
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
            getenforce,
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
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
        }

        if self.modules_path.exists() {
            let mut modules = Vec::new();
            if let OtherOk(entries_count_dir) = fs::read_dir(self.modules_path.clone()) {
                let entries_count = entries_count_dir.count();
                if let OtherOk(entries) = fs::read_dir(self.modules_path.clone()) {
                    if entries_count > 0 {
                        for entry in entries {
                            if let OtherOk(entry) = entry {
                                let path = entry.path();
                                modules.push(
                                    path.file_name()
                                        .expect(&msg_err_str("Failed to get file name."))
                                        .to_string_lossy()
                                        .to_string(),
                                );
                            }
                        }
                        msg_regular(&format!("Modules: {}", modules.len().to_string().blue()));
                        for i in modules {
                            if PathBuf::from(self.modules_path.clone())
                                .join(i.clone())
                                .join("disable")
                                .exists()
                            {
                                msg_sub(&format!("{} (disabled)", i));
                            } else {
                                msg_sub(&i);
                            }
                        }
                    } else {
                        msg_regular("No modules installed.");
                    }
                }
            }
        } else {
            msg_regular("No modules installed.");
        }
        Ok(())
    }

    fn is_module_disabled(&mut self, module: &str) -> anyhow::Result<bool> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
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
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
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
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
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
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
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
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
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
        let args = format!("magisk --install-module {}", waydroid_module_path_string);
        msg_main(&format!("Installing magisk module"));
        msg_sub("Copying to temporary directory");
        fs::copy(module_path.clone(), tmp_dir.join(filename.to_string()))?;
        msg_sub("Installing");
        let (success, _) = waydroid_su(if self.getenforce {
            vec!["sh", "-c", &args]
        } else {
            vec!["magisk", "--install-module", &waydroid_module_path_string]
        })?;
        waydroid_su(vec!["rm", &waydroid_module_path.to_string_lossy()])?;
        if !success {
            msg_err("Installation failed!");
        } else {
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
        }

        Ok(())
    }

    pub fn cmd(&mut self, applet: &str, args: Vec<&str>) -> anyhow::Result<String> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
        }
        let mut args_new: Vec<&str> = Vec::with_capacity(args.len() + 1);
        args_new.push("magisk");
        args_new.push(&applet);
        args_new.extend(args.iter().map(|s| s));
        let (success, out) = if self.getenforce {
            let mut args_str = String::new();
            for item in args_new.clone() {
                if args_str.is_empty() {
                    args_str = item.to_string();
                } else {
                    args_str = format!("{} {}", args_str, item);
                }
            }
            waydroid_su(vec!["sh", "-c", &args_str])?
        } else {
            waydroid_su(args_new)?
        };
        if success && applet != "--sqlite" {
            if !out.is_empty() {
                println!("{}", out.bold());
            }
        }
        Ok(out)
    }

    pub fn sqlite(&mut self, arg: &str) -> anyhow::Result<String> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
        }
        Ok(self.cmd("--sqlite", vec![arg])?)
    }

    pub fn get_zygisk(&mut self) -> anyhow::Result<bool> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
        }
        let zygisk_str = self.sqlite("\"SELECT value FROM settings WHERE key == 'zygisk'\"")?;

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
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
        }
        if enabled {
            self.sqlite("\"REPLACE INTO settings (key,value) VALUES('zygisk',1)\"")?;
        } else {
            self.sqlite("\"REPLACE INTO settings (key,value) VALUES('zygisk',0)\"")?;
        }
        Ok(())
    }

    pub fn status(&mut self) -> anyhow::Result<()> {
        if !self.waydroid.is_container_running()? {
            return Err(anyhow!("Waydroid container isn't running!"));
        }
        if !self.installed {
            return Err(anyhow!("KitsuneMagisk isn't installed!"));
        }

        let mut args = Vec::new();
        args.push("pidof");
        args.push("magiskd");
        let daemon_running = if self.getenforce {
            waydroid_su(vec!["sh", "-c", "pidof magiskd"])?
        } else {
            waydroid_su(args)?
        };
        let daemon_running_str;
        if daemon_running.0 {
            daemon_running_str = "Running".blue();
        } else {
            daemon_running_str = "Stopped".red();
        }

        let zygisk = self.get_zygisk()?;
        let zygisk_str = if zygisk { "Yes".blue() } else { "No".red() };

        msg_regular(&format!("Daemon: {}", daemon_running_str));
        msg_regular(&format!("Installed: {}", self.version.blue()));
        msg_regular(&format!("Zygisk: {}", zygisk_str));
        Ok(())
    }
}
