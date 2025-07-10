# Scope

This is a (very) simple oscilloscope coded in Rust using [NIH-plug](https://github.com/robbert-vdh/nih-plug).

### Building

These instructions are for OSX.

```bash
cargo xtask bundle scope --release
cp -r ./target/bundled/scope.clap /Library/Audio/Plug-Ins/CLAP
```
