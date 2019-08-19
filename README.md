# ow-arcade-bot

## Update the lambda
cargo build --release --target x86_64-unknown-linux-musl --bin bootstrap
zip -j rust.zip ./target/x86_64-unknown-linux-musl/release/bootstrap

## Update the config file
```bash
export OWARCADEBOT_S3_BUCKET=foo
export OWARCADEBOT_S3_KEY_CONFIG=cfg.json
export OWARCADEBOT_S3_KEY_GAMESTATE=owarcadebot/gamestate.json
cargo run --bin ow-arcade-cli -- -v config pull > cfg.json
vim cfg.json
cargo run --bin ow-arcade-cli -- -v config push cfg.json
```
