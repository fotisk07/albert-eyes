run:
  @cargo fmt
  @cargo check
  @cargo run

deploy:
    @cargo build --release --target aarch64-unknown-linux-gnu
    @ssh albert 'systemctl --user stop albert-eyes-display.service'
    @scp target/aarch64-unknown-linux-gnu/release/albert-eyes albert:~/.local/bin/albert-eyes
    @ssh albert 'systemctl --user start albert-eyes-display.service'
