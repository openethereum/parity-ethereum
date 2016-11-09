#!/bin/bash

mkdir -p $HOME/Library/LaunchAgents
cat > $HOME/Library/LaunchAgents/io.parity.ethereum.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>io.parity.ethereum</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/libexec/parity</string>
        <string>--warp</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$HOME/.parity/log.out</string>
    <key>StandardErrorPath</key>
    <string>$HOME/.parity/log.err</string>
</dict>
</plist>
EOF

mkdir -p $HOME/.parity/906a34e69aec8c0d
echo -n '{"fat_db":false,"mode":"passive","mode.alarm":3600,"mode.timeout":300,"pruning":"fast","tracing":false}' > $HOME/.parity/906a34e69aec8c0d/user_defaults

chown -R $USER $HOME/.parity $HOME/Library/LaunchAgents $HOME/Library/LaunchAgents/io.parity.ethereum.plist

su $USER -c "launchctl load $HOME/Library/LaunchAgents/io.parity.ethereum.plist"
sleep 1

open http://127.0.0.1:8080/

