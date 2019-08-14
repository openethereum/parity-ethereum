#!/bin/bash
set -e

target_filename="parity-${CI_COMMIT_TAG:-${CI_COMMIT_REF_NAME}}.exe"
apt -y update
apt -y install ftp

echo "__________Publish Windows binaries to Avast Whitelisting program__________"

ftp -pinv whitelisting.avast.com <<EOF
quote USER ftp_parityio
quote PASS $avast_ftp_password
cd /share
put ./artifacts/x86_64-pc-windows-msvc/parity.exe $target_filename
bye
EOF

echo "__________Publish Windows binaries to Kaspersky Whitelisting program__________"

ftp -pinv whitelist1.kaspersky-labs.com <<EOF
quote USER wl-ParityTech
quote PASS $kaspersky_ftp_password
put ./artifacts/x86_64-pc-windows-msvc/parity.exe $target_filename
bye
EOF
