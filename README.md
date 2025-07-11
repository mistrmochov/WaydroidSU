<div align="center">

# WaydroidSU

**WaydroidSU** is a CLI Magisk manager and installer for Waydroid!

</div>

- [WaydroidSU](#waydroidsu)
  - [Introduction](#introduction)
  - [Notice](#notice)
  - [Installation](#installation)
    - [Notice](#notice-1)
  - [Building](#building)
  - [Usage](#usage)
    - [Installation of KitsuneMagisk using WaydroidSU](#installation-of-kitsunemagisk-using-waydroidsu)
      - [Notice](#notice-2)
    - [Updating Magisk using WaydroidSU](#updating-magisk-using-waydroidsu)
  - [SELinux - fully supported!](#selinux---fully-supported)
  - [Compatibility](#compatibility)

## Introduction

I have forked the latest KitsuneMagisk and made small patches, So It could work on Waydroid once again. (https://github.com/mistrmochov/KitsuneMagisk-Waydroid)

As a follow up to this, I decided to develop full Magisk CLI installer and manager for Waydroid in Rust. Big credits goes to @nitanmarcel as this project has been inspired by his project `waydroid-magisk`.

## Notice

Keep in mind, that this project is still in early stages and there might be some unexpected issues.

## Installation

You can install WaydroidSU by installing packages from [releases](https://github.com/mistrmochov/WaydroidSU/releases)

Choose your package accordingly for your distro and your architecture.

Download your selected package and use your package manager to install it.

Ubuntu:
```bash
sudo apt install ./wsu-0.1.0-1-x86_64-ubuntu_24+.deb
```

Fedora:
```bash
sudo dnf install ./wsu-0.1.0-1-x86_64-fc42.rpm
```

Arch:
```bash
sudo pacman -U ./wsu-0.1.0-1-x86_64-archlinux.pkg.tar.zst
```

SUSE:
```bash
sudo zypper in ./wsu-0.1.0-1-x86_64-suse.rpm
```

Alpine:
```bash
sudo apk add ./wsu-0.1.0-1-x86_64-alpine.apk
```

If your distro isn't in releases you will have to build it yourself. I would still recommend you to build the program yourself always, it will make your life easier when updating and you will have always the latest build.

### Notice

If you are using SUSE you might get a warning from zypper, that the package is not signed, you can ignore this message and continue by selecting `i`, also this package has been built on SUSE Tumbleweed.

## Building

Main building dependencies are `make` and `cargo`. This project has been made on Rust version `1.88.0`, if your cargo version from your package manager isn't compatible with this project, please install cargo using rustup or install rust manually from the official website: https://www.rust-lang.org/tools/install

Dependencies
- Ubuntu/Debian: `libdbus-1-dev`, `libssl-dev`, `pkg-config`, `build-essential`
- Fedora: `dbus-devel`, `openssl-devel`, `pkgconf-pkg-config`
- Arch: `dbus`, `openssl`, `base-devel`
- SUSE: `dbus-1-devel`, `libopenssl-devel`, `pkgconf`
- Alpine: `dbus-dev`, `openssl-dev`, `pkgconf`, `build-base`

Runtime dependencies
- Ubuntu/Debian: `liblzma5`, `libbz2-1.0`, `libssl3`, `libdbus-1-3`
- Fedora: `xz-libs`, `bzip2-libs`, `openssl`, `dbus-libs`
- Arch: `xz`, `bzip2`, `openssl`, `dbus`
- SUSE: `liblzma5`, `libbz2-1`, `libopenssl3`, `libdbus-1-3`
- Alpine: `xz-libs`, `bzip2`, `libssl3`, `libcrypto3`, `dbus-libs`

Run the following commmands to build and install WaydroidSU:

```bash
git clone https://github.com/mistrmochov/WaydroidSU
cd WaydroidSU
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

If your Waydroid was running before the installation, it will start automatically after running install command, but if it wasn't you'll have to start your Waydroid manually, but the program will guide you in that regard. Keep in mind, that Waydroid will be booting much longer with Magisk installed.

After your Waydroid boots app, run this command to run additional setup or upgrade the Magisk stub to full manager by clicking on it and then proceed to the additional setup.

```bash
sudo wsu setup
```

Now after Waydroid restarts, KitsuneMagisk is fully installed.

#### Notice

Please do NOT install Magisk through the Magisk manager app (`Direct install into system`)! It might break your setup as it uses a different installation process and you might need to reinstall Magisk!

### Updating Magisk using WaydroidSU

Run following command to check/install update.

```bash
sudo wsu update
```

## SELinux - fully supported!

I have managed to get this working even on devices with SELinux enforcing enabled!

## Compatibility

At this point, this project only supports systems with 64 bits architecture.

## Issues

Some magisk modules in KitsuneMagisk might cause that Play Store cannot be loaded, it's basically stuck on the Play Store logo.

So in my case, module `tricky_store` has been causing this issue. You can fix this issue by enabling `sulist` in `magiskhide` through the magisk manager or via WaydroidSU.