#!/usr/bin/env bash
set -euo pipefail

target="aarch64-unknown-linux-gnu"
remote_path="/tmp/albert-eyes-prototype-$$"

cleanup() {
  ssh albert "rm -f '$remote_path'" >/dev/null 2>&1 || true
}
trap cleanup EXIT

cargo build --target "$target"
scp "target/$target/debug/albert-eyes" "albert:$remote_path"
ssh -t albert "$remote_path"
