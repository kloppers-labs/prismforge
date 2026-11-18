# PrismForge

Desktop RGB control on top of a running [OpenRGB](https://openrgb.org) SDK server (protocol 6, OpenRGB 1.0+).

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Live probe against a local OpenRGB server (read-only unless `--device` and `--color` are both given):

```sh
cargo run -p prismforge-openrgb --example probe -- --host 127.0.0.1 --port 6742
cargo run -p prismforge-openrgb --example probe -- --device <ref from listing> --color ff8000 [--zone 0]
```
