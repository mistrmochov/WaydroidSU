use crate::magisk::Magisk;
use crate::magisk_files::get_status;
use anyhow::Ok;
use colored::*;
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
// pub fn msg_regular_str(msg: &str) -> String {
//     format!("{}", msg.bold())
// }

pub fn print_modules(mut magisk: Magisk) -> anyhow::Result<()> {
    let modules = magisk.get_list_modules()?;
    let path = magisk.modules_path.clone();
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

pub fn print_superuser(mut magisk: Magisk) -> anyhow::Result<()> {
    let superuser_list = magisk.get_superuser_list()?;
    for (pkg, verdict) in &superuser_list {
        if superuser_list[0] == (pkg.clone(), verdict) {
            msg_regular("Superuser:");
        }
        msg_sub(&format!(
            "{} | {}",
            pkg,
            if verdict == &"allowed" {
                verdict.blue()
            } else {
                verdict.red()
            }
        ));
    }
    Ok(())
}

pub fn print_status() -> anyhow::Result<()> {
    let (daemon_running, version, zygisk) = get_status()?;
    let daemon_running_str = if daemon_running {
        "Running".blue()
    } else {
        "Stopped".red()
    };
    let (version_str, zygisk_str) = if daemon_running {
        (
            version.blue(),
            if zygisk { "Yes".blue() } else { "No".red() },
        )
    } else {
        ("No".red(), "N/A".red())
    };

    msg_regular(&format!("Daemon: {}", daemon_running_str));
    msg_regular(&format!("Installed: {}", version_str));
    msg_regular(&format!("Zygisk: {}", zygisk_str));
    Ok(())
}
