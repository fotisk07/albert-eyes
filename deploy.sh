#!/usr/bin/env bash
set -euo pipefail


cargo build --release --target aarch64-unknown-linux-gnu

scp target/aarch64-unknown-linux-gnu/release/albert-eyes albert:~/.local/bin/albert-eyes.new
