clippy:
	cd ./bililive-core && cargo clippy
	cd ./bililive && cargo clippy
	cd ./actix-bililive && cargo clippy

clippy-pedantic:
	cd ./bililive-core && cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery
	cd ./bililive && cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery
	cd ./actix-bililive && cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery -A clippy::future_not_send

test $FAST_TEST="1":
	cd ./bililive-core && cargo test
	cd ./bililive-core && cargo test --no-default-features --features async-std-support
	cd ./bililive && cargo test
	cd ./bililive && cargo test --no-default-features --features async-native-tls
	cd ./actix-bililive && cargo test

test-full:
	cd ./bililive-core && cargo test
	cd ./bililive-core && cargo test --no-default-features --features async-std-support
	cd ./bililive && cargo test
	cd ./bililive && cargo test --no-default-features --features async-native-tls
	cd ./actix-bililive && cargo test

doc crate:
	cd "./{{crate}}" && cargo doc --all-features

doc-all: (doc "bililive-core") (doc "bililive") (doc "actix-bililive")

open crate: (doc crate)
	xdg-open "./target/doc/{{replace(crate, "-", "_")}}/index.html"