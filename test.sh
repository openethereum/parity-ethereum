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


validate () {
  if [ "$VALIDATE" -eq "1" ]
  then
    echo "________Validate build________"
    time cargo check $@ --no-default-features
    time cargo check $@ --manifest-path util/io/Cargo.toml --no-default-features
    time cargo check $@ --manifest-path util/io/Cargo.toml --features "mio"
  
    # Validate chainspecs
    echo "________Validate chainspecs________"
    time ./scripts/validate_chainspecs.sh
  else
    echo "# not validating due to \$VALIDATE!=1"
  fi
}

cpp_test () {
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
}

cargo_test () {
  echo "________Running Parity Full Test Suite________"
  git submodule update --init --recursive
  time cargo test $OPTIONS --features "$FEATURES" --all $@ -- --test-threads $THREADS
}


if [ "$CARGO_TARGET" ]
then
  validate --target $CARGO_TARGET
else
  validate
fi

test "${RUN_TESTS}" = "true" && cpp_test

if [ "$CARGO_TARGET" ]
then

  if [ "${RUN_TESTS}" = "true" ]
  then
    cargo_test --target $CARGO_TARGET $@
  else
    cargo_test --no-run --target $CARGO_TARGET $@
  fi
else
  cargo_test $@
fi

