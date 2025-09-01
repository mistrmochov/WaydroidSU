use crate::cli::*;
use crate::install::{install, remove, setup, update};
use crate::magisk::Magisk;
use crate::print::{msg_err, msg_regular, print_modules, print_status, print_superuser};
use crate::utils::{
    command_exists, get_arch, is_mounted_at, is_waydroid_initialized, root, umount_system,
};
use anyhow::{Ok, anyhow};
use clap::Parser;
use std::env::temp_dir;
use std::result::Result::Ok as OtherOk;

mod cli;
mod constants;
mod container;
mod install;
mod magisk;
mod magisk_files;
mod print;
mod selinux;
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

fn kitsune_or_err(magisk: &Magisk, applet: &str) -> anyhow::Result<()> {
    if !magisk.version().contains("kitsune") && !magisk.version().contains("v27.2-Waydroid") {
        return Err(anyhow!(format!(
            "{} - Is only available for Kitsune",
            applet
        )));
    }
    Ok(())
}

fn magisk_or_err(magisk: &Magisk, applet: &str) -> anyhow::Result<()> {
    if magisk.version().contains("kitsune") || magisk.version().contains("v27.2-Waydroid") {
        return Err(anyhow!(format!(
            "{} - Is only available for Magisk",
            applet
        )));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    try_run_or_exit!(preflight());

    match cli.command {
        Commands::Status => {
            try_run!(print_status());
        }
        Commands::Install(args) => {
            let (arch, arch_supported) = get_arch();
            if arch_supported {
                let apk_path = args.apk.unwrap_or_else(|| "".to_string());
                if let Err(e) = install(arch, &apk_path, false, args.kitsune) {
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
                ModuleCommand::List => try_run!(print_modules(magisk)),
                ModuleCommand::Install(arg) => try_run!(magisk.install_module(&arg.module)),
                ModuleCommand::Remove(arg) => try_run!(magisk.remove_module(&arg.module)),
                ModuleCommand::Disable(arg) => try_run!(magisk.disable_module(&arg.module)),
                ModuleCommand::Enable(arg) => try_run!(magisk.enable_module(&arg.module)),
            }
        }
        Commands::Magiskhide { command } => {
            let mut magisk = magisk_or_exit!();
            try_run_or_exit!(kitsune_or_err(&magisk, "magiskhide"));
            match command {
                MagiskhideCommand::Status => {
                    try_run!(magisk.cmd("magiskhide", vec!["status"], false))
                }
                MagiskhideCommand::Enable => {
                    try_run!(magisk.cmd("magiskhide", vec!["enable"], false))
                }
                MagiskhideCommand::Disable => {
                    try_run!(magisk.cmd("magiskhide", vec!["disable"], false))
                }
                MagiskhideCommand::Sulist { command: Some(sub) } => match sub {
                    SulistSubCommand::Enable => {
                        try_run!(magisk.cmd("magiskhide", vec!["sulist", "enable"], false))
                    }
                    SulistSubCommand::Disable => {
                        try_run!(magisk.cmd("magiskhide", vec!["sulist", "disable"], false))
                    }
                },
                MagiskhideCommand::Sulist { command: None } => {
                    try_run!(magisk.cmd("magiskhide", vec!["sulist"], false))
                }
                MagiskhideCommand::Ls => try_run!(magisk.cmd("magiskhide", vec!["ls"], false)),
                MagiskhideCommand::Add(arg) => {
                    try_run!(magisk.cmd("magiskhide", vec!["add", &arg.pkg], false))
                }
                MagiskhideCommand::Rm(arg) => {
                    try_run!(magisk.cmd("magiskhide", vec!["rm", &arg.pkg], false))
                }
            }
        }
        Commands::Denylist { command } => {
            let mut magisk = magisk_or_exit!();
            try_run_or_exit!(magisk_or_err(&magisk, "denylist"));
            match command {
                DenylistCommand::Status => {
                    try_run!(magisk.cmd("--denylist", vec!["status"], false))
                }
                DenylistCommand::Enable => {
                    try_run!(magisk.cmd("--denylist", vec!["enable"], false))
                }
                DenylistCommand::Disable => {
                    try_run!(magisk.cmd("--denylist", vec!["disable"], false))
                }
                DenylistCommand::Ls => try_run!(magisk.cmd("--denylist", vec!["ls"], false)),
                DenylistCommand::Add(arg) => {
                    try_run!(magisk.cmd("--denylist", vec!["add", &arg.pkg], false))
                }
                DenylistCommand::Rm(arg) => {
                    try_run!(magisk.cmd("--denylist", vec!["rm", &arg.pkg], false))
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
        Commands::Superuser { command } => {
            let mut magisk = magisk_or_exit!();
            match command {
                SuperuserCommand::Allow(arg) => try_run!(magisk.superuser_manage(&arg.pkg, true)),
                SuperuserCommand::Deny(arg) => try_run!(magisk.superuser_manage(&arg.pkg, false)),
                SuperuserCommand::List => try_run!(print_superuser(magisk)),
            }
        }
    }

    Ok(())
}
