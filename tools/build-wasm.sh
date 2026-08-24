#!/bin/sh
# Build every lesson crate to wasm and stage the binaries into public/.
# Pins rustup's toolchain explicitly: Homebrew's rustc shadows it in PATH
# on this machine and ships no cross-targets.
set -e
export RUSTC="$(rustup which rustc)"
cargo build --target wasm32-unknown-unknown --release -p lesson-two-mirrors
cp target/wasm32-unknown-unknown/release/lesson_two_mirrors.wasm \
   public/lessons/two-mirrors/lesson.wasm
echo "staged: $(wc -c < public/lessons/two-mirrors/lesson.wasm) bytes"
