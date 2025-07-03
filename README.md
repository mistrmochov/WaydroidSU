<div align="center">

# WaydroidSU

**WaydroidSU** is a CLI Magisk manager and installer for Waydroid!

</div>

- [WaydroidSU](#waydroidsu)
  - [Introduction](#introduction)
  - [Building](#building)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Installation of KitsuneMagisk using WaydroidSU](#installation-of-kitsunemagisk-using-waydroidsu)
      - [Notice](#notice)
    - [Updating Magisk using WaydroidSU](#updating-magisk-using-waydroidsu)
  - [SELinux - fully supported!](#selinux---fully-supported)

## Introduction

I have forked the latest KitsuneMagisk and made small patches, So It could work on Waydroid once again. (https://github.com/mistrmochov/KitsuneMagisk-Waydroid)

As a follow up to this, I decided to develop full Magisk CLI installer and manager for Waydroid in Rust. Big credits goes to @nitanmarcel as this project has been inspired by his project `waydroid-magisk`.

## Building

Main building dependencies are `make` and `cargo`. This project has been made on Rust version `1.88.0`, if your cargo version from your package manager isn't compatible with this project, please install cargo using rustup or install rust manually from the official website: https://www.rust-lang.org/tools/install

Dependencies
- Ubuntu/Debian: `libdbus-1-dev`, `libssl-dev`, `pkg-config`, `build-essential`
- Fedora: `dbus-devel`, `openssl-devel`, `pkgconf-pkg-config`
- Arch: `dbus`, `openssl`, `base-devel`
- SUSE: `dbus-1-devel`, `libopenssl-devel`, `pkgconf`
- Alpine: `dbus-dev`, `openssl-dev`, `pkgconf`

## Installation

As of now only way to install this project is by building it, but packages will be avialable soon in releases.

Run the following commmands to build and install WaydroidSU:

```bash
git clone https://github.com/mistrmochov/WaydroidSU
make
sudo make install
```
This will place WaydroidSU binary in `/usr/bin`.

If you want to clean build files run:
```bash
make clean
```

If you wish to uninstall WaydroidSU run the following command in the repo's directory:

```bash
sudo make uninstall
```

## Usage

* Go to [USAGE.md](https://github.com/mistrmochov/WaydroidSU/blob/main/docs/USAGE.md) for complete list of avialable commands or use `-h / --help`.

### Installation of KitsuneMagisk using WaydroidSU

**Your Waydroid has to be initialized!**

By default Magisk is installed to Waydroid's overlay system, however if your Waydroid has `mount_overlays` disabled, the program will ask you if you want to install Magisk directly into the system image. (Not recommended)

```bash
sudo wsu install # Normal installation
sudo wsu install --apk /path/to/apk # Not recommended since this is the only version, that will work on Waydroid
```

If your Waydroid was running before the installation, it will start automatically after running install command, but if it wasn't you'll have to start your Waydroid manually, but the program will guide you in that regard. Keep in mind, that Waydroid will be booting longer with Magisk installed.

After your Waydroid boots app, run this command to run additional setup or upgrade the Magisk stub to full manager by clicking on it and then proceed to the additional setup.

```bash
sudo wsu setup
```

Now after Waydroid restarts, KitsuneMagisk is fully installed.

#### Notice

Please do not install Magisk in Magisk manager app! It might break your setup as it uses a different installation process and you might need to reinstall Magisk!

### Updating Magisk using WaydroidSU

Run following command to check/install update.

```bash
sudo wsu update
```

## SELinux - fully supported!

I have managed to get this working even on devices with SELinux enforcing enabled!