# `script` phase: you usually build, test and generate docs in this phase

set -ex

# NOTE Workaround for rust-lang/rust#31907 - disable doc tests when cross compiling
# This has been fixed in the nightly channel but it would take a while to reach the other channels
disable_cross_doctests() {
  local host

  case "$TRAVIS_OS_NAME" in
    linux)
      host=x86_64-unknown-linux-gnu
      ;;
    osx)
      host=x86_64-apple-darwin
      ;;
  esac

  if [ "$host" != "$TARGET" ] && [ "$CHANNEL" != "nightly" ]; then
    if [ "$TRAVIS_OS_NAME" = "osx" ]; then
      brew install gnu-sed --default-names
    fi

    find src -name '*.rs' -type f | xargs sed -i -e 's:\(//.\s*```\):\1 ignore,:g'
  fi
}

# TODO modify this function as you see fit
# PROTIP Always pass `--target $TARGET` to cargo commands, this makes cargo output build artifacts
# to target/$TARGET/{debug,release} which can reduce the number of needed conditionals in the
# `before_deploy`/packaging phase
run_test_suite() {
  case $TARGET in
    # use an emulator to run the cross compiled binaries
    arm-unknown-linux-gnueabihf)
      # build tests but don't run them
      cargo test --target $TARGET --no-run

      # run tests in emulator
      find target/$TARGET/debug -maxdepth 1 -executable -type f \
        -exec qemu-arm -L /usr/arm-linux-gnueabihf '{}' ';'

      # build the main executable
      cargo build --target $TARGET

      # run the main executable using the emulator
      qemu-arm -L /usr/arm-linux-gnueabihf target/$TARGET/debug/hello
      ;;
    *)
      cargo build --target $TARGET --verbose
      cargo run --target $TARGET
      cargo test --target $TARGET
      ;;
  esac

  # sanity check the file type
  file target/$TARGET/debug/hello
}

main() {
  disable_cross_doctests
  run_test_suite
}
