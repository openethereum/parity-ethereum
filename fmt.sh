#!/bin/sh

RUSTFMT="rustfmt --write-mode overwrite"

$RUSTFMT ./ethash/src/lib.rs
$RUSTFMT ./ethcore/src/lib.rs
$RUSTFMT ./evmjit/src/lib.rs
$RUSTFMT ./json/src/lib.rs
$RUSTFMT ./miner/src/lib.rs
$RUSTFMT ./parity/main.rs
$RUSTFMT ./rpc/src/lib.rs
$RUSTFMT ./webapp/src/lib.rs
$RUSTFMT ./sync/src/lib.rs
$RUSTFMT ./util/src/lib.rs

