#!/usr/bin/env bash

FLAGS="-Xlint:deprecation"
PARITY_JAVA="../../Parity.java"
# parity-clib must be built with feature `jni` in debug-mode to work
PARITY_LIB=".:../../../target/debug/"

# build
cd ..
cargo build --features jni
cd -
javac $FLAGS -d $PWD $PARITY_JAVA
javac $FLAGS *.java
# Setup the path `libparity.so` and run
java -Djava.library.path=$PARITY_LIB Main
