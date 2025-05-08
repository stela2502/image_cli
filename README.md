# image_cli

My Rust programs are bundled into an Apptainer image for later transfere into a secure HPC environment.
This program is a way to automatically add a help system into this image.

It assumes that all cusom programs are available as executables in /usr/local/bin and help strings are collected from all these programs and rwitten to the command line.

This in the end should enable for a simple to maintain help system in an Apptainer based group of programs.

# Install

After havin installed the Rust compiler you can install the software with

```bash
cargo install --git https://github.com/stela2502/image_cli
```

Or if installing it for all users:

```bash
CARGO_INSTALL_ROOT=/usr/local cargo install --git https://github.com/stela2502/image_cli
```

# Usage

```
image_cli
```


