<div align="center">

# WaydroidSU

**WaydroidSU** is a CLI Magisk manager and installer for Waydroid!

</div>

- [WaydroidSU](#waydroidsu)
  - [Introduction](#introduction)
    - [Update](#update)
  - [Installation](#installation)
    - [Notice](#notice)
  - [Building](#building)
  - [Usage](#usage)
    - [Installation of Magisk using WaydroidSU](#installation-of-magisk-using-waydroidsu)
      - [Notice](#notice-1)
    - [Updating Magisk using WaydroidSU](#updating-magisk-using-waydroidsu)
  - [SELinux - fully supported!](#selinux---fully-supported)
  - [Compatibility](#compatibility)
  - [Issues](#issues)
    - [Play Store issue (Kitsune only)](#play-store-issue-kitsune-only)
    - [Session as root issue](#session-as-root-issue)

## Introduction

I have forked the latest KitsuneMagisk and made small patches, So It could work on Waydroid once again. (https://github.com/mistrmochov/KitsuneMagisk-Waydroid)

As a follow up to this, I decided to develop full Magisk CLI installer and manager for Waydroid in Rust. Big credits goes to @nitanmarcel as this project has been inspired by his project `waydroid-magisk`.

### Update

So I have created a new Magisk fork directly from topjohnwu (https://github.com/mistrmochov/MagiskForWaydroid), because Kitsune is pretty outdated and I personally don't like the direction where this project is going. So now you will have two options, you can stick to Kitsune or you can install my new modern fork of Magisk. It was a little tricky to get this working, because upstream Magisk doesn't have --setup-sbin like Kitsune has, but the result is worth it!

I made a similar patches I did to Kitsune, but I also had to make zygisk working as the official built in zygisk doesn't work on Waydroid. I basically implemented ReZygisk module directly into the Magisk itself, you as user won't see any difference at all. You will simply enable or disable zygisk in the settings as usual.

## Installation

You can install WaydroidSU by installing packages from [releases](https://github.com/mistrmochov/WaydroidSU/releases)

Choose your package accordingly for your distro and your architecture.

Download your selected package and use your package manager to install it.

Ubuntu:
```bash
sudo apt install ./wsu-0.1.1-1-x86_64-ubuntu_24+.deb
```

Fedora:
```bash
sudo dnf install ./wsu-0.1.1-1-x86_64-fc42.rpm
```

Arch:
```bash
sudo pacman -U ./wsu-0.1.1-1-x86_64-archlinux.pkg.tar.zst
```

SUSE:
```bash
sudo zypper in ./wsu-0.1.1-1-x86_64-suse.rpm
```

Alpine:
```bash
sudo apk add ./wsu-0.1.1-1-x86_64-alpine.apk
```

If your distro isn't in releases you will have to build it yourself. I would still recommend you to build the program yourself always, it will make your life easier when updating and you will have always the latest build.

### Notice

If you are using SUSE you might get a warning from zypper, that the package is not signed, you can ignore this message and continue by selecting `i`, also the package for SUSE has been built on SUSE Tumbleweed.

## Building

Main building dependencies are `make` and `cargo`. This project has been made on Rust version `1.88.0`, if your cargo version from your package manager isn't compatible with this project, please install cargo using rustup or install rust manually from the official website: https://www.rust-lang.org/tools/install

Dependencies
- Ubuntu/Debian: `libdbus-1-dev`, `libssl-dev`, `pkg-config`, `build-essential`
- Fedora: `dbus-devel`, `openssl-devel`, `pkgconf-pkg-config`
- Arch: `dbus`, `openssl`, `base-devel`, `pkgconf`
- SUSE: `dbus-1-devel`, `libopenssl-devel`, `pkgconf`
- Alpine: `dbus-dev`, `openssl-dev`, `pkgconf`, `build-base`

Runtime dependencies
- Ubuntu/Debian: `liblzma5`, `libbz2-1.0`, `libssl3`, `libdbus-1-3`, `libsystemd0`, `libcap2`
- Fedora: `xz-libs`, `bzip2-libs`, `openssl`, `dbus-libs`, `systemd-libs`, `libcap`
- Arch: `xz`, `bzip2`, `openssl`, `dbus`, `systemd-libs`, `libcap`
- SUSE: `liblzma5`, `libbz2-1`, `libopenssl3`, `libdbus-1-3`, `libsystemd0`, `libcap2`
- Alpine: `xz-libs`, `bzip2`, `libssl3`, `libcrypto3`, `dbus-libs`, `libelogind`, `libcap`

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

### Installation of Magisk using WaydroidSU

**Your Waydroid has to be initialized!**

By default Magisk is installed to Waydroid's overlay system, however if your Waydroid has `mount_overlays` disabled, the program will ask you if you want to install Magisk directly into the system image. (Not recommended)

```bash
sudo wsu install # Normal installation
sudo wsu install --new # My new modern fork of Magisk
sudo wsu install --apk /path/to/apk # Not recommended since this is the only version, that will work on Waydroid
```

It is now required for Waydroid to be running before the installation! Keep in mind, that Waydroid will be booting much longer with Magisk installed.

After your Waydroid boots app, run this command to run additional setup or upgrade the Magisk stub to full manager by clicking on it and then proceed to the additional setup.

```bash
sudo wsu setup
```

Now after Waydroid restarts, Magisk is fully installed.

#### Notice

Please do NOT install Magisk through the Magisk manager app (`Direct install into system`)! It might break your setup as it uses a different installation process and you might need to reinstall Magisk!

This is of course only the case for Kitsune as Magisk does not have such an option.

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

### Play Store issue (Kitsune only)

Some magisk modules in KitsuneMagisk might cause that Play Store cannot be loaded, it's basically stuck on the Play Store logo.

So in my case, module `tricky_store` has been causing this issue. You can fix this issue by enabling `sulist` in `magiskhide` through the magisk manager or via WaydroidSU.

### Session as root issue

If you are using root as your default main account (logged in as a root user through your login screen), Magisk will not work.

Magisk will be able to install, but `su` requests will get automatically rejected, there will be likely more issues around this thing.