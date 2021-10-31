# bililive-core

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PhotonQuantum/bililive-rs/Test?style=flat-square)](https://github.com/PhotonQuantum/bililive-rs/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/bililive-core?style=flat-square)](https://crates.io/crates/bililive-core)
[![Documentation](https://img.shields.io/docsrs/bililive-core?style=flat-square)](https://docs.rs/bililive-core)

A simple stream-based bilibili live danmaku implementation for Rust.

This crate contains core traits, types and parsing implementations needed to build a
complete bilibili live client.

If you need a batteries-included client, you may want to look at `bililive` or `actix-bililive`.

## Feature Flags
- `tokio-support` (default) - enable tokio support.
- `async-std-support` - enable async-std support.
- `not-send` - Remove `Send` constraints on traits and types. Useful for actix clients.