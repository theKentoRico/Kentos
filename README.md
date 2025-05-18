# Kentos

Kentos is a FOSS OS I'm working on.

# Building Kentos

## Prerequisites:

1. Cargo

2. Rust (nightly)

3. Something to run Kentos on (actual device, VM, etc.)

## How to

Run
`cargo bootimage --target x86_64-thekentorico-kentos.json` and run `target/x86_64-thekentorico-kentos.json/debug/bootimage-kentos.bin` in your preferred VM or boot it in a real device.