# `before_deploy` phase: here we package the build artifacts

set -ex

mktempd() {
  echo $(mktemp -d 2>/dev/null || mktemp -d -t tmp)
}

# Generate artifacts for release
mk_artifacts() {
  cargo build --target $TARGET --release
}

mk_tarball() {
  # create a "staging" directory
  local temp_dir=$(mktempd)
  local out_dir=$(pwd)

  # TODO update this part to copy the artifacts that make sense for your project
  # NOTE All Cargo build artifacts will be under the 'target/$TARGET/{debug,release}'
  cp target/$TARGET/release/hello $temp_dir

  pushd $temp_dir

  # release tarball will look like 'rust-everywhere-v1.2.3-x86_64-unknown-linux-gnu.tar.gz'
  tar czf $out_dir/${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}.tar.gz *

  popd $temp_dir
  rm -r $temp_dir
}

main() {
  mk_artifacts
  mk_tarball
}

main
