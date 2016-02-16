#!/bin/sh

RUSTFMT="rustfmt --write-mode overwrite"

$RUSTFMT ./util/src/lib.rs
$RUSTFMT ./ethcore/src/lib.rs
$RUSTFMT ./ethash/src/lib.rs
$RUSTFMT ./rpc/src/lib.rs
$RUSTFMT ./sync/src/lib.rs
$RUSTFMT ./evmjit/src/lib.rs
$RUSTFMT ./parity/main.rs

