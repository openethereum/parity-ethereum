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
    time cargo check $@ --frozen --no-default-features
    time cargo check $@ --frozen --manifest-path util/io/Cargo.toml --no-default-features
    time cargo check $@ --frozen --manifest-path util/io/Cargo.toml --features "mio"

    # Validate chainspecs
    echo "________Validate chainspecs________"
    time ./scripts/validate_chainspecs.sh
  else
    echo "# not validating due to \$VALIDATE!=1"
  fi
}

cpp_test () {
  case $CARGO_TARGET in
    (x86_64-unknown-linux-gnu)
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
      ;;
    (*)
      echo "________Skipping the C++ example________"
      ;;
  esac
}

cargo_test () {
  echo "________Running Parity Full Test Suite________"
  git submodule update --init --recursive
  time cargo test $OPTIONS --features "$FEATURES" --frozen --all $@ -- --test-threads $THREADS
}


if [ "$CARGO_TARGET" ]
then
  validate --target $CARGO_TARGET
else
  validate
fi

test "${RUN_TESTS}" = "all" && cpp_test

if [ "$CARGO_TARGET" ]
then

  case "${RUN_TESTS}" in
    (cargo|all)
      cargo_test --target $CARGO_TARGET $@
      ;;
    ('')
      cargo_test --no-run --target $CARGO_TARGET $@
      ;;
  esac
else
  cargo_test $@
fi
