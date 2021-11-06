ci: fmt clippy

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy

clippy-pedantic:
	cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery

test $FAST_TEST="1":
	cargo test
	cargo test --no-default-features --features async-native-tls

test-full:
	cargo test
	cargo test --no-default-features --features async-native-tls

doc:
	cargo doc --all-features

doc-all: doc

open: doc
	xdg-open "./target/doc/bililive/index.html"