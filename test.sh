#!/bin/sh
# Running Parity Full Test Suite

FEATURES="json-tests,ci-skip-issue"
OPTIONS="--release"
VALIDATE=1
THREADS=8

case $1 in
  --no-json)
    FEATURES="ipc"
    shift # past argument=value
    ;;
  --no-release)
    OPTIONS=""
    shift
    ;;
  --no-validate)
    VALIDATE=0
    shift
    ;;
  --no-run)
    OPTIONS="--no-run"
    shift
    ;;
  *)
    # unknown option
    ;;
esac

set -e



case $CARGO_TARGET in
  (x86_64-unknown-linux-gnu|x86_64-apple-darwin|x86_64-pc-windows-msvc|'')
    # native builds
    if [ "$VALIDATE" -eq "1" ]
    then
      echo "________Validate build________"
      time cargo check --no-default-features
      time cargo check --manifest-path util/io/Cargo.toml --no-default-features
      time cargo check --manifest-path util/io/Cargo.toml --features "mio"
    
      # Validate chainspecs
      echo "________Validate chainspecs________"
      time ./scripts/validate_chainspecs.sh
    fi


    # Running the C++ example
    echo "________Running the C++ example________"
    cd parity-clib-examples/cpp && \
      mkdir -p build && \
      cd build && \
      cmake .. && \
      make -j $THREADS && \
      ./parity-example && \
      cd .. && \
      rm -rf build && \
      cd ../..

    # Running tests
    echo "________Running Parity Full Test Suite________"
    git submodule update --init --recursive
    time cargo test  $OPTIONS --features "$FEATURES" --all $@ -- --test-threads $THREADS
    ;;
  (*)
    if [ "$VALIDATE" -eq "1" ]
    then
      echo "________Validate build________"
      time cargo check --target $CARGO_TARGET --no-default-features
      time cargo check --target $CARGO_TARGET --manifest-path util/io/Cargo.toml --no-default-features
      time cargo check --target $CARGO_TARGET --manifest-path util/io/Cargo.toml --features "mio"
    
      # Validate chainspecs
      echo "________Validate chainspecs________"
      time ./scripts/validate_chainspecs.sh
    fi

    # Per default only build but not run the tests
    echo "________Building Parity Full Test Suite________"
    git submodule update --init --recursive
    time cargo test  --no-run --target $CARGO_TARGET $OPTIONS --features "$FEATURES" --all $@ -- --test-threads $THREADS
    ;;
esac

