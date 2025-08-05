use clap::{Args, Parser, Subcommand};
#[derive(Parser)]
#[command(
    name = "wsu",
    version,
    about = "CLI Magisk manager for Waydroid",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Show Magisk status")]
    Status,
    #[command(about = "Install Magisk")]
    Install(InstallArgs),
    #[command(about = "Run additional setup for Magisk")]
    Setup,
    #[command(about = "Remove Magisk")]
    Remove,
    #[command(about = "Update Magisk")]
    Update,
    #[command(about = "Manage magisk modules")]
    Module {
        #[command(subcommand)]
        command: ModuleCommand,
    },
    #[command(about = "Manage MagiskHide (Kitsune)")]
    Magiskhide {
        #[command(subcommand)]
        command: MagiskhideCommand,
    },
    #[command(about = "Manage Denylist (Magisk)")]
    Denylist {
        #[command(subcommand)]
        command: DenylistCommand,
    },
    #[command(about = "Manage Zygisk")]
    Zygisk {
        #[command(subcommand)]
        command: ZygiskCommand,
    },
}

#[derive(Args)]
pub struct InstallArgs {
    #[arg(long, help = "Path to target apk (optional)")]
    pub apk: Option<String>,
    #[arg(short, long, help = "Modern version of Magisk")]
    pub new: bool,
}

#[derive(Subcommand)]
pub enum MagiskhideCommand {
    #[command(about = "Show MagiskHide status")]
    Status,
    #[command(about = "Show SuList status / Manage SuList")]
    Sulist {
        #[command(subcommand)]
        command: Option<SulistSubCommand>,
    },
    #[command(about = "Enable MagiskHide")]
    Enable,
    #[command(about = "Disable MagiskHide")]
    Disable,
    #[command(about = "Add target to hidelist/sulist")]
    Add(AddRemoveArgs),
    #[command(about = "Remove target from hidelist/sulist")]
    Rm(AddRemoveArgs),
    #[command(about = "List targets in hidelist/sulist")]
    Ls,
}

#[derive(Subcommand)]
pub enum DenylistCommand {
    #[command(about = "Show Denylist status")]
    Status,
    #[command(about = "Enable Denylist")]
    Enable,
    #[command(about = "Disable Denylist")]
    Disable,
    #[command(about = "Add target to Denylist")]
    Add(AddRemoveArgs),
    #[command(about = "Remove target from Denylist")]
    Rm(AddRemoveArgs),
    #[command(about = "List targets in Denylist")]
    Ls,
}

#[derive(Args)]
pub struct AddRemoveArgs {
    #[arg(help = "Target package")]
    pub pkg: String,
}

#[derive(Subcommand)]
pub enum SulistSubCommand {
    #[command(about = "Enable SuList")]
    Enable,
    #[command(about = "Disable SuList")]
    Disable,
}

#[derive(Subcommand)]
pub enum ModuleCommand {
    #[command(about = "List installed magisk modules")]
    List,
    #[command(about = "Remove magisk module")]
    Remove(ModuleRemoveArg),
    #[command(about = "Install magisk module")]
    Install(ModuleInstallArg),
    #[command(about = "Disable magisk module")]
    Disable(ModuleDisableEnableArg),
    #[command(about = "Enable magisk module")]
    Enable(ModuleDisableEnableArg),
}

#[derive(Subcommand)]
pub enum ZygiskCommand {
    #[command(about = "Show Zygisk status")]
    Status,
    #[command(about = "Enable Zygisk")]
    Enable,
    #[command(about = "Disable Zygisk")]
    Disable,
}

#[derive(Args)]
pub struct ModuleInstallArg {
    #[arg(help = "Path to target magisk module")]
    pub module: String,
}

#[derive(Args)]
pub struct ModuleRemoveArg {
    #[arg(help = "Name of the target magisk module")]
    pub module: String,
}

#[derive(Args)]
pub struct ModuleDisableEnableArg {
    #[arg(help = "Name of the target magisk module")]
    pub module: String,
}
