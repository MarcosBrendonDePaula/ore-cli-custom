# ORE CLI

A command line interface for ORE cryptocurrency mining.

## Install

To install the CLI, use [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```sh
cargo install ore-cli
```

## Build

To build the codebase from scratch, checkout the repo and use cargo to build:

```sh
cargo build --release
```

## Help

You can use the `-h` flag on any command to pull up a help menu with documentation:

```sh
ore -h
```

## New Argument

The `--min-difficulty` argument was previously controlled by Hardhat, but now you can directly specify the minimum difficulty expected to solve through the CLI. The difficulty range goes from 3 to 35.

### Example Command

```sh
./ore --keypair id.json --min-difficulty 10
```