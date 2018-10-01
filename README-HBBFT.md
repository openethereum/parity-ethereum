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

### Signer Nodes

FIXME: Complete me

`curl --data '{"jsonrpc":"2.0","method":"personal_sendTransaction","params":[{"from":"0x002eb83d1d04ca12fe1956e67ccaa195848e437f","to":"0x00Bd138aBD70e2F00903268F3Db08f2D25677C9e","value":"0x10000"}, "richie"],"id":0}' -H "Content-Type: application/json" -X POST localhost:8500`

### Status

Experimental