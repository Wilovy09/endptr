default:
    cargo run

c:
    cargo check --all-targets --all-features

f:
    cargo fmt --all

fc:
    cargo clippy --all-targets --all-features -- -D warnings

i:
    cargo install --path .