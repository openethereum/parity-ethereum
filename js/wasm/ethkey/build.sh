# Remove previous build to avoid name conflicts
rm -rf target/wasm32-unknown-emscripten/*

# Build using nightly rustc + emscripten
rustup run nightly cargo build --release --target=wasm32-unknown-emscripten

# Copy final WASM file over
cp ./target/wasm32-unknown-emscripten/release/deps/parity_ethkey_wasm-*.wasm ./ethkey.wasm

# Create a Base64-encoded JS version of the wasm file for easy inclusion in Webpack
node base64ify

# Copy Base64-encoded JS version to src
cp ./ethkey.wasm.js ../../packages/api/local/ethkey/ethkey.wasm.js

# rm -f ./ethkey.wasm ./ethkey.opt.wasm ./ethkey.wasm.js
