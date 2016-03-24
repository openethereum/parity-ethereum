# `install` phase: install stuff needed for the `script` phase

set -ex

mktempd() {
  echo $(mktemp -d 2>/dev/null || mktemp -d -t tmp)
}

install_multirust() {
  local temp_dir=$(mktempd)

  git clone https://github.com/brson/multirust $temp_dir

  pushd $temp_dir
  ./build.sh
  ./install.sh --prefix=~/multirust

  multirust default $CHANNEL
  rustc -V
  cargo -V

  popd
  rm -rf $temp_dir
}

install_standard_crates() {
  local host
  case "$TRAVIS_OS_NAME" in
    linux)
      host=x86_64-unknown-linux-gnu
      ;;
    osx)
      host=x86_64-apple-darwin
      ;;
  esac

  if [ "$host" != "$TARGET" ]; then
    if [ "$CHANNEL" = "nightly" ]; then
      multirust add-target nightly $TARGET
    else
      local version
      if [ "$CHANNEL" = "stable" ]; then
        # e.g. 1.6.0
        version=$(rustc -V | cut -d' ' -f2)
      else
        version=beta
      fi

      local tarball=rust-std-${version}-${TARGET}

      local temp_dir=$(mktempd)
      curl -s https://static.rust-lang.org/dist/${tarball}.tar.gz | \
        tar --strip-components 1 -C $temp_dir -xz

      $temp_dir/install.sh --prefix=$(rustc --print sysroot)

      rm -r $temp_dir
    fi
  fi
}

configure_cargo() {
  local prefix=
  case "$TARGET" in
    arm*-gnueabihf)
      prefix=arm-linux-gnueabihf
      ;;
    *)
      return
      ;;
  esac

  # information about the cross compiler
  $prefix-gcc -v

  # tell cargo which linker to use for cross compilation
  mkdir -p .cargo
  cat >>.cargo/config <<EOF
[target.$TARGET]
linker = "$prefix-gcc"
EOF
}

main() {
  install_multirust
  install_standard_crates
  configure_cargo

  # TODO if you need to install extra stuff add it here
}

main
