# androidx-release-watcher

Rust binary that polls Google's Maven repository and extracts the latest version of all dependencies, with as few network calls as possible.

## Installation

### Using `cargo`

Run `cargo install adx` on a terminal.

### From source

Clone this repository, then run `cargo install --path .` in the directory you cloned to.

## Usage

### Filtering packages by name
```
$ adx enterprise
androidx.enterprise:enterprise-feedback:1.0.0-rc01
androidx.enterprise:enterprise-feedback-testing:1.0.0-rc01
```

### Getting detailed information on packages

```
$ adx -d appcompat
Group ID: androidx.appcompat
Artifact ID: appcompat
Available versions: 1.2.0-alpha01, 1.1.0, 1.1.0-rc01, 1.1.0-beta01, 1.1.0-alpha05, 1.1.0-alpha04, 1.1.0-alpha03, 1.1.0-alpha02, 1.1.0-alpha01, 1.0.2, 1.0.1, 1.0.0, 1.0.0-rc02, 1.0.0-rc01, 1.0.0-beta01, 1.0.0-alpha3, 1.0.0-alpha1
Latest: androidx.appcompat:appcompat:1.2.0-alpha01

Group ID: androidx.appcompat
Artifact ID: appcompat-resources
Available versions: 1.2.0-alpha01, 1.1.0, 1.1.0-rc01, 1.1.0-beta01, 1.1.0-alpha05, 1.1.0-alpha04, 1.1.0-alpha03
Latest: androidx.appcompat:appcompat-resources:1.2.0-alpha01
```

### Getting latest version of all packages

```
$ adx --all
androidx.media2:media2:1.0.0-alpha04
androidx.media2:media2-exoplayer:1.0.2
androidx.media2:media2-player:1.0.2
androidx.media2:media2-common:1.0.2
androidx.media2:media2-session:1.0.2
...
```

## TODO

- [ ] Allow filtering by channel
