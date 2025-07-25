use crate::cli::*;
use crate::constants::WAYDROID_CONFIG;
use crate::container::WaydroidContainer;
use crate::magisk::Magisk;
use crate::magisk_files::{install, remove, setup, update};
use crate::utils::{get_arch, is_mounted_at, umount_system};
use anyhow::{Ok, anyhow};
use clap::Parser;
use colored::*;
use std::env;
use std::env::temp_dir;
use std::path::PathBuf;
use std::process::Command;
use std::result::Result::Ok as OtherOk;
use which::which;

mod cli;
mod constants;
mod container;
mod magisk;
mod magisk_files;
mod utils;

macro_rules! magisk_or_exit {
    () => {
        match Magisk::new() {
            OtherOk(m) => m,
            Err(e) => {
                msg_err(&e.to_string());
                return Ok(());
            }
        }
    };
}

macro_rules! try_run {
    ($expr:expr) => {
        if let Err(e) = $expr {
            msg_err(&e.to_string());
        }
    };
}

macro_rules! try_run_or_exit {
    ($expr:expr) => {
        if let Err(e) = $expr {
            msg_err(&e.to_string());
            return Ok(());
        }
    };
}

fn is_waydroid_initialized() -> bool {
    PathBuf::from(WAYDROID_CONFIG).exists()
}

fn command_exists(cmd: &str) -> bool {
    which(cmd).is_ok()
}

pub fn root() -> bool {
    let user_out = Command::new("bash")
        .args(["-c", "whoami"])
        .output()
        .expect(&msg_err_str("Failed to execute \"whoami\" command."));
    let user = String::from_utf8_lossy(&user_out.stdout).trim().to_string();

    user == "root"
}

pub fn get_data_home() -> anyhow::Result<String> {
    fn xdg_data_home() -> anyhow::Result<String> {
        let waydroid = WaydroidContainer::new()?;
        let session = waydroid.get_session();
        if !session.is_empty() {
            for (key, value) in session {
                if key == "xdg_data_home" {
                    return Ok(value);
                }
            }
        }
        Ok("".to_string())
    }
    if let OtherOk(sudo_home) = env::var("SUDO_HOME") {
        if !sudo_home.contains("root") {
            return Ok(PathBuf::from(sudo_home)
                .join(".local/share")
                .to_string_lossy()
                .to_string());
        }
    } else {
        return Ok(xdg_data_home()?);
    }
    Ok("".to_string())
}

pub fn msg_err(msg: &str) {
    eprintln!("{}: {}", "error".red().bold(), msg);
}

pub fn msg_err_str(msg: &str) -> String {
    format!("{}: {}", "error".red().bold(), msg)
}

pub fn msg_main(msg: &str) {
    println!("[{}] {}", "WSU".blue().bold(), msg.bold());
}

pub fn msg_sub(msg: &str) {
    println!(" {}", msg);
}

pub fn msg_end(msg: &str) {
    println!("\n{}", msg.bold());
}

pub fn msg_regular(msg: &str) {
    println!("{}", msg.bold());
}
pub fn msg_regular_str(msg: &str) -> String {
    format!("{}", msg.bold())
}

fn preflight() -> anyhow::Result<()> {
    if !command_exists("waydroid") {
        return Err(anyhow!("Waydroid is not installed on your system!"));
    }
    if !is_waydroid_initialized() {
        return Err(anyhow!("Your Waydroid is not initialized!"));
    }
    if !root() {
        return Err(anyhow!("Needs to be ran as sudo!"));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    try_run_or_exit!(preflight());

    match cli.command {
        Commands::Status => {
            let mut magisk = magisk_or_exit!();
            try_run!(magisk.status());
        }
        Commands::Install(args) => {
            let (arch, arch_supported) = get_arch();
            if arch_supported {
                let apk_path = args.apk.unwrap_or_else(|| "".to_string());
                if let Err(e) = install(arch, &apk_path, false) {
                    msg_err(&e.to_string());
                    try_run!(remove(true, false));
                }
            }
        }
        Commands::Setup => try_run!(setup()),
        Commands::Remove => {
            if let Err(e) = remove(false, false) {
                msg_err(&e.to_string());
                let mnt = temp_dir().join("waydroidsu/mnt");
                let mounted = match is_mounted_at(&mnt.to_string_lossy()) {
                    OtherOk(m) => m,
                    Err(e) => {
                        msg_err(&e.to_string());
                        return Ok(());
                    }
                };
                if mounted {
                    try_run!(umount_system(false));
                }
            }
        }
        Commands::Update => {
            let (arch, arch_supported) = get_arch();
            if arch_supported {
                try_run!(update(arch));
            }
        }
        Commands::Module { command } => {
            let mut magisk = magisk_or_exit!();
            match command {
                ModuleCommand::List => try_run!(magisk.list_modules()),
                ModuleCommand::Install(arg) => try_run!(magisk.install_module(&arg.module)),
                ModuleCommand::Remove(arg) => try_run!(magisk.remove_module(&arg.module)),
                ModuleCommand::Disable(arg) => try_run!(magisk.disable_module(&arg.module)),
                ModuleCommand::Enable(arg) => try_run!(magisk.enable_module(&arg.module)),
            }
        }
        Commands::Magiskhide { command } => {
            let mut magisk = magisk_or_exit!();
            match command {
                MagiskhideCommand::Status => try_run!(magisk.cmd("magiskhide", vec!["status"])),
                MagiskhideCommand::Enable => try_run!(magisk.cmd("magiskhide", vec!["enable"])),
                MagiskhideCommand::Disable => try_run!(magisk.cmd("magiskhide", vec!["disable"])),
                MagiskhideCommand::Sulist { command: Some(sub) } => match sub {
                    SulistSubCommand::Enable => {
                        try_run!(magisk.cmd("magiskhide", vec!["sulist", "enable"]))
                    }
                    SulistSubCommand::Disable => {
                        try_run!(magisk.cmd("magiskhide", vec!["sulist", "disable"]))
                    }
                },
                MagiskhideCommand::Sulist { command: None } => {
                    try_run!(magisk.cmd("magiskhide", vec!["sulist"]))
                }
                MagiskhideCommand::Ls => try_run!(magisk.cmd("magiskhide", vec!["ls"])),
                MagiskhideCommand::Add(arg) => {
                    try_run!(magisk.cmd("magiskhide", vec!["add", &arg.pkg]))
                }
                MagiskhideCommand::Rm(arg) => {
                    try_run!(magisk.cmd("magiskhide", vec!["rm", &arg.pkg]))
                }
            }
        }
        Commands::Zygisk { command } => {
            let mut magisk = magisk_or_exit!();
            match command {
                ZygiskCommand::Status => {
                    let zygisk = match magisk.get_zygisk() {
                        OtherOk(z) => z,
                        Err(e) => {
                            msg_err(&e.to_string());
                            return Ok(());
                        }
                    };
                    if zygisk {
                        msg_regular("Zygisk is enabled");
                    } else {
                        msg_regular("Zygisk is disabled");
                    }
                }
                ZygiskCommand::Enable => try_run!(magisk.set_zygisk(true)),
                ZygiskCommand::Disable => try_run!(magisk.set_zygisk(false)),
            }
        }
    }

    Ok(())
}
