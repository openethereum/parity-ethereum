#!/usr/bin/env bash

FLAGS="-Xlint:deprecation"
PARITY_JAVA="../../parity-clib/Parity.java"
PARITY_LIB=".:../../target/debug/"

# build
javac $FLAGS -d $PWD $PARITY_JAVA
javac $FLAGS *.java
# Setup the path `libparity.so` and run
java -Djava.library.path=$PARITY_LIB Main
