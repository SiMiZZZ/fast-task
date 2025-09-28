default: check


lint:
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::style -D clippy::perf

clippy: lint


fmt:
    cargo fmt

check: fmt lint
