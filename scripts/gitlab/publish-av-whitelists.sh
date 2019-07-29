#!/bin/bash
set -e

echo "__________Publish Windows binaries to Avast Whitelisting program__________"

target_filename="parity-${CI_COMMIT_TAG:-${CI_COMMIT_REF_NAME}}.exe"
apt -y update
apt -y install ftp
ftp -pinv whitelisting.avast.com <<EOF
quote USER ftp_parityio
quote PASS $avast_ftp_password
cd /share
put ./artifacts/x86_64-pc-windows-msvc/parity.exe $target_filename
bye
EOF

