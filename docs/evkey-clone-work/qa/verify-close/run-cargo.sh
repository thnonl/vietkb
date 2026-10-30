#!/usr/bin/env bash
# QA round-close verify (2026-10-30). Serial cargo steps; one raw file per step plus exit-codes.txt.
# No vietkb.exe is launched, no input is sent, --live-hooks is never passed.
cd /e/vietkb || exit 1
D=docs/evkey-clone-work/qa/verify-close
R=$D/raw
: > $R/exit-codes.txt
run() { name=$1; shift; ( "$@" ) > "$R/$name.txt" 2>&1; code=$?; echo "$name EXIT=$code CMD=$*" >> $R/exit-codes.txt; }
run 01-cargo-test-workspace cargo test --workspace
run 02-vkb-windows-e2e-harness cargo test -p vkb-windows --features e2e-harness
run 03a-vkb-core-no_alloc cargo test -p vkb-core --test no_alloc
run 03b-vkb-core-raw_zone cargo test -p vkb-core --test raw_zone
run 03c-vkb-core-tone_change cargo test -p vkb-core --test tone_change
run 04a-build-vkb-cli-debug cargo build -p vkb-cli
run 04b-build-vkb-cli-release cargo build -p vkb-cli --release
echo DONE >> $R/exit-codes.txt
