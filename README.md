# androidx-release-watcher [![No Maintenance Intended](http://unmaintained.tech/badge.svg)](http://unmaintained.tech/) [![Built with Garnix](https://img.shields.io/endpoint?url=https%3A%2F%2Fgarnix.io%2Fapi%2Fbadges%2Fmsfjarvis%2Fadx%3Fbranch%3Dmain)](https://garnix.io)

Rust binary that polls Google's Maven repository and finds the latest version of the requested dependencies.

## Installation

### Using `cargo`

Run `cargo install --locked adx` on a terminal.

### From source

```shell
git clone https://github.com/msfjarvis/adx
cd adx
cargo install --path adx
```

## Usage

### Find the latest release of a package

```shell
$ adx appcompat
androidx.appcompat:appcompat:1.3.0-alpha02
androidx.appcompat:appcompat-resources:1.3.0-alpha02
```

### Find the latest stable version of a package

```shell
$ adx --channel stable appcompat
androidx.appcompat:appcompat:1.2.0
androidx.appcompat:appcompat-resources:1.2.0
```
