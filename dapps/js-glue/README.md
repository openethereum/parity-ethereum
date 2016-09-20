# Parity Dapps (JS-glue)

Code generator to simplify creating a built-in Parity Dapp

# How to create new builtin Dapp.
1. Clone this repository.

   ```bash
   $ git clone https://github.com/ethcore/parity.git
   ```

1. Create a new directory for your Dapp. (`./myapp`)

   ```bash
   $ mkdir -p ./parity/dapps/myapp/src/web
   ```

1. Copy your frontend files to `./dapps/myapp/src/web` (bundled ones)

   ```bash
   $ cp -r ./myapp-src/* ./parity/dapps/myapp/src/web
   ```

1. Instead of creating `web3` in your app. Load (as the first script tag in `head`):

   ```html
   <script src="/parity-utils/inject.js"></script>
   ```

   The `inject.js` script will create global `web3` instance with proper provider that should be used by your dapp.

1. Create `./parity/dapps/myapp/Cargo.toml` with you apps details. See example here: [parity-status Cargo.toml](https://github.com/ethcore/parity-ui/blob/master/status/Cargo.toml).

   ```bash
   $ git clone https://github.com/ethcore/parity-ui.git
   $ cd ./parity-ui/
   $ cp ./home/Cargo.toml ../parity/dapps/myapp/Cargo.toml
   $ cp ./home/build.rs ../parity/dapps/myapp/build.rs
   $ cp ./home/src/lib.rs ../parity/dapps/myapp/src/lib.rs
   $ cp ./home/src/lib.rs.in ../parity/dapps/myapp/src/lib.rs.in
   # And edit the details of your app
   $ vim ../parity/dapps/myapp/Cargo.toml # Edit the details
   $ vim ./parity/dapps/myapp/src/lib.rs.in # Edit the details
   ```
# How to include your Dapp into `Parity`?
1. Edit `dapps/Cargo.toml` and add dependency to your application (it can be optional)

   ```toml
   # Use git repo and version
   parity-dapps-myapp = { path="./myapp" }
   ```

1. Edit `dapps/src/apps.rs` and add your application to `all_pages` (if it's optional you need to specify two functions - see `parity-dapps-wallet` example)

1. Compile parity.

   ```bash
   $ cargo build --release # While inside `parity`
   ```

1. Commit the results.

   ```bash
   $ git add myapp && git commit -am "My first Parity Dapp".
   ```
