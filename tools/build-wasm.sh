#!/bin/sh
# Build every lesson crate to wasm and stage the binaries into public/.
# Pins rustup's toolchain explicitly: Homebrew's rustc shadows it in PATH
# on this machine and ships no cross-targets.
set -e
# A consistent toolchain end to end: Homebrew rust shadows rustup on this
# machine, and MIXING them (RUSTC alone) breaks clippy with E0514.
export PATH="$(dirname "$(rustup which rustc)"):$PATH"
LESSONS=$(ls -d lessons/*/crate | sed 's|lessons/||; s|/crate||')
for L in $LESSONS; do
  CRATE="lesson-$(echo $L)"
  cargo build --target wasm32-unknown-unknown --release -p "$CRATE"
  WASM="target/wasm32-unknown-unknown/release/$(echo $CRATE | tr - _).wasm"
  cp "$WASM" "public/lessons/$L/lesson.wasm"
  echo "staged $L: $(wc -c < public/lessons/$L/lesson.wasm) bytes"
done
