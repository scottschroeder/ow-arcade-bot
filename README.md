# ow-arcade-bot

## Update the lambda
```bash
cargo build --release --target x86_64-unknown-linux-musl --bin bootstrap
zip -j rust.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
```

## Update the config file
```bash
export OWARCADEBOT_DISCORD_TOKEN=xxx
export AWS_ACCESS_KEY_ID=xxx
export AWS_SECRET_ACCESS_KEY=xxx
export AWS_DEFAULT_REGION=us-west-2
export OWARCADEBOT_S3_BUCKET=foo
export OWARCADEBOT_S3_KEY_CONFIG=cfg.json
export OWARCADEBOT_S3_KEY_GAMESTATE=owarcadebot/gamestate.json

cargo run --bin ow-arcade-cli -- -v config pull > cfg.json
vim cfg.json
cargo run --bin ow-arcade-cli -- -v config validate cfg.json
cargo run --bin ow-arcade-cli -- -v watcher -c cfg.json
cargo run --bin ow-arcade-cli -- -v config push cfg.json
```
