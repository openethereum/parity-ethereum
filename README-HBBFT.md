# parity-hbbft

An experimental peer-to-peer client using the [Honey Badger Byzantine Fault
Tolerant consensus algorithm](https://github.com/poanetwork/hbbft).

## Usage

### Running a Test Peer

1. `git clone https://github.com/poanetwork/parity-ethereum`
2. `cd parity-ethereum`
3. `git checkout hbbft`
3. `./indica-node 0`

#### Additional Peers

Create a network by running additional peers:

1. Open a new terminal window or tab (use ctrl-pgup/pgdown to cycle between
   tabs quickly).
2. `cd {...}/parity-ethereum`
3. `./indica-node 1`
4. (Repeat 1 and 2), `./indica-node 2`, `./indica-node 3`, `./indica-node 4`,
   etc. Currently, only 3 nodes are required to start a network.

Each peer will generate a number of random transactions at regular intervals,
process them accordingly, and output complete batches. If your terminal is
spammed with batch outputs, consensus is working.

Type `./indica-node 0 --help` or `cargo run -- --help` for command line options.

See the
[`run-node`](https://github.com/poanetwork/hydrabadger/blob/master/run-node)
script for additional optional environment variables that can be set. To turn
on debug log output: `export HYDRABADGER_LOG_ADDTL=debug` and/or `echo "export
HYDRABADGER_LOG_ADDTL=debug" >> ~/.profile`.

### Status

Experimental