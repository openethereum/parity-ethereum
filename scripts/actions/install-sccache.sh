#!/usr/bin/pwsh

$SCCACHE_CACHE_SIZE="1G"
$SCCACHE_IDLE_TIMEOUT=0
$version = "0.2.12"
$platform =
  @{ "macOS"   = "x86_64-apple-darwin"
     "Linux"   = "x86_64-unknown-linux-musl"
     "Windows" = "x86_64-pc-windows-msvc"
   }.${{ runner.os }}
$basename = "sccache-$version-$platform"
$url = "https://github.com/mozilla/sccache/releases/download/" +
       "$version/$basename.tar.gz"
curl -LO $url
tar -xzvf "$basename.tar.gz"
. $basename/sccache --start-server
echo "::add-path::$(pwd)/$basename"
echo "::set-env name=RUSTC_WRAPPER::sccache"
