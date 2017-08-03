#Fuzzing crates with cargo-fuzz

##Installation
Install cargo-fuzz using cargo:
  - `cargo install cargo-fuzz`

##Initializing fuzz tests
Change into the target crate's base directory, e.g.:
  - `cd parity/util/rlp`

Initialize the fuzz directory:
  - `cargo fuzz init`
  - `cargo fuzz init -t named_first_test`

Fuzz tests stored under `./fuzz/fuzz_targets/*.rs`

##Adding tests
Add a test named `anyname` under `fuzz/fuzz_targets/anyname.rs`:
  - `cargo fuzz add anyname`

This also adds an entry to `fuzz/Cargo.toml` for creating a binary target
with the test's name, e.g.
```
[[bin]]
name = "anyname"
path = "fuzz_targets/anyname.rs"
```

##Editing tests
Make changes and import any necessary libraries to test target features
  - `vim ./fuzz/fuzz_targets/untrusted.rs`

##Running tests
While still in the crate's root directory, run the following:
  - `cargo fuzz run <test-name>`
  - `cargo fuzz run -j <num-jobs> <test-name>` for parallell jobs

Examples and detailed information on libfuzzer output can be found [here](https://rust-fuzz.github.io/book/cargo-fuzz/tutorial.html)

Efficient fuzzing tips from Google: [efficient fuzzing](https://chromium.googlesource.com/chromium/src/+/master/testing/libfuzzer/efficient_fuzzer.md)
