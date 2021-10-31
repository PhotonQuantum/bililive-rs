# bililive-rs

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PhotonQuantum/bililive-rs/Test?style=flat-square)](https://github.com/PhotonQuantum/bililive-rs/actions/workflows/test.yml)

Simple stream-based bilibili live client libraries.

## Crates

### Core

- [bililive-core](bililive-core) - Core traits and structs for a simple stream-based bilibili live danmaku implementation.

### Implementations

- [bililive](bililive) - A simple stream-based bilibili live client library backed by [async-tungstenite](https://github.com/sdroege/async-tungstenite). Supports both tokio and async-std.
- [actix-bililive](actix-bililive) - A simple stream-based bilibili live client library for the Actix ecosystem, backed by [awc](https://github.com/actix/actix-web/tree/master/awc).

## Features

- Ergonomic `Stream`/`Sink` interface.
- Easy establishment of connection via given live room id.
- Handles heartbeat packets automatically.
- Auto retry when connection fails (optional).
- Decompresses `Zlib` payloads automatically.

## License

This project is licensed under [MIT License](LICENSE).