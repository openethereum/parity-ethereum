#!/usr/bin/env pwsh
$os=$args[0]
$version="0.2.12"
echo "Current OS:" $os
switch ($os){
   "macOS" {$platform = "x86_64-apple-darwin"}
   "Linux" {$platform = "x86_64-unknown-linux-musl"}
   "Windows"  {$platform ="x86_64-pc-windows-msvc"}
}
echo "Target arch: " $platform
$basename = "sccache-$version-$platform"
$url = "https://github.com/mozilla/sccache/releases/download/"+"$version/$basename.tar.gz"
echo "Download sccache from "+$url
curl -LO $url
tar -xzvf "$basename.tar.gz"
ls $basename/
. $basename/sccache --start-server
echo "::add-path::$(pwd)/$basename"
echo "::set-env name=RUSTC_WRAPPER::sccache"
