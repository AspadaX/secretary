#! /bin/sh

set -e

cargo run --example async
cargo run --example async_force
cargo run --example sync
cargo run --example sync_force
