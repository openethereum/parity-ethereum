 // build.rs

// Bring in a dependency on an externally maintained `gcc` package which manages
// invoking the C compiler.
extern crate gcc;

fn main() {
    gcc::compile_library("libtinykeccak.a", &["src/tinykeccak.c"]);
}

