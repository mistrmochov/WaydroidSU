- [Usage](#usage)
  - [status](#status)
  - [install](#install)
  - [setup](#setup)
  - [remove](#remove)
  - [update](#update)
  - [module](#module)
    - [module list](#module-list)
    - [module remove](#module-remove)
    - [module install](#module-install)
    - [module disable](#module-disable)
    - [module enable](#module-enable)
  - [magiskhide](#magiskhide)
    - [magiskhide status](#magiskhide-status)
    - [magiskhide sulist](#magiskhide-sulist)
      - [magiskhide sulist enable](#magiskhide-sulist-enable)
      - [magiskhide sulist disable](#magiskhide-sulist-disable)
    - [magiskhide enable](#magiskhide-enable)
    - [magiskhide disable](#magiskhide-disable)
    - [magiskhide add](#magiskhide-add)
    - [magiskhide rm](#magiskhide-rm)
    - [magiskhide ls](#magiskhide-ls)
  - [zygisk](#zygisk)
    - [zygisk status](#zygisk-status)
    - [zygisk enable](#zygisk-enable)
    - [zygisk disable](#zygisk-disable)


# Usage

## status

```
Show Magisk status

Usage: wsu status

Options:
  -h, --help  Print help
```

## install

```
Install KitsuneMagisk

Usage: wsu install [OPTIONS]

Options:
      --apk <APK>  Path to target apk (optional)
  -h, --help       Print help
```

## setup

```
Run additional setup for KitsuneMagisk

Usage: wsu setup

Options:
  -h, --help  Print help
```

## remove

```
Remove KitsuneMagisk

Usage: wsu remove

Options:
  -h, --help  Print help
```

## update

```
Update KitsuneMagisk

Usage: wsu update

Options:
  -h, --help  Print help
```

## module

```
Manage magisk modules

Usage: wsu module <COMMAND>

Commands:
  list     List installed magisk modules
  remove   Remove magisk module
  install  Install magisk module
  disable  Disable magisk module
  enable   Enable magisk module

Options:
  -h, --help  Print help
```

### module list

```
List installed magisk modules

Usage: wsu module list

Options:
  -h, --help  Print help
```

### module remove

```
Remove magisk module

Usage: wsu module remove <MODULE>

Arguments:
  <MODULE>  Name of the target magisk module

Options:
  -h, --help  Print help
```

### module install

```
Install magisk module

Usage: wsu module install <MODULE>

Arguments:
  <MODULE>  Path to target magisk module

Options:
  -h, --help  Print help
```

### module disable

```
Disable magisk module

Usage: wsu module disable <MODULE>

Arguments:
  <MODULE>  Name of the target magisk module

Options:
  -h, --help  Print help
```

### module enable

```
Enable magisk module

Usage: wsu module enable <MODULE>

Arguments:
  <MODULE>  Name of the target magisk module

Options:
  -h, --help  Print help
```

## magiskhide

```
Manage MagiskHide

Usage: wsu magiskhide <COMMAND>

Commands:
  status   Show MagiskHide status
  sulist   Show SuList status / Manage SuList
  enable   Enable MagiskHide
  disable  Disable MagiskHide
  add      Add target to hidelist/sulist
  rm       Remove target from hidelist/sulist
  ls       List targets in hidelist/sulist

Options:
  -h, --help  Print help
```

### magiskhide status

```
Show MagiskHide status

Usage: wsu magiskhide status

Options:
  -h, --help  Print help
```

### magiskhide sulist

```
Show SuList status / Manage SuList

Usage: wsu magiskhide sulist [COMMAND]

Commands:
  enable   Enable SuList
  disable  Disable SuList

Options:
  -h, --help  Print help
```

#### magiskhide sulist enable

```
Enable SuList

Usage: wsu magiskhide sulist enable

Options:
  -h, --help  Print help
```

#### magiskhide sulist disable

```
Disable SuList

Usage: wsu magiskhide sulist disable

Options:
  -h, --help  Print help
```

### magiskhide enable

```
Enable MagiskHide

Usage: wsu magiskhide enable

Options:
  -h, --help  Print help
```

### magiskhide disable

```
Disable MagiskHide

Usage: wsu magiskhide disable

Options:
  -h, --help  Print help
```

### magiskhide add

```
Add target to hidelist/sulist

Usage: wsu magiskhide add <PKG>

Arguments:
  <PKG>  Target package

Options:
  -h, --help  Print help
```

### magiskhide rm

```
Remove target from hidelist/sulist

Usage: wsu magiskhide rm <PKG>

Arguments:
  <PKG>  Target package

Options:
  -h, --help  Print help
```

### magiskhide ls

```
List targets in hidelist/sulist

Usage: wsu magiskhide ls

Options:
  -h, --help  Print help
```

## zygisk

```
Manage Zygisk

Usage: wsu zygisk <COMMAND>

Commands:
  status   Show Zygisk status
  enable   Enable Zygisk
  disable  Disable Zygisk

Options:
  -h, --help  Print help
```

### zygisk status

```
Show Zygisk status

Usage: wsu zygisk status

Options:
  -h, --help  Print help
```

### zygisk enable

```
Enable Zygisk

Usage: wsu zygisk enable

Options:
  -h, --help  Print help
```

### zygisk disable

```
Disable Zygisk

Usage: wsu zygisk disable

Options:
  -h, --help  Print help
```