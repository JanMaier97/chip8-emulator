#!/bin/sh
cargo build --target x86_64-pc-windows-gnu &&
cp target/x86_64-pc-windows-gnu/debug/chip8-emulator.exe . &&
exec ./chip8-emulator.exe "$@"
