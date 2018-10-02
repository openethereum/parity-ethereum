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

Running a node with an associated engine signer account allows the node to
participate in proof-of-authority-style consensus algorithms. The
`indica-node-signer` script works exactly like the  `indica-node` script but
requires a bit of set up to add the associated test accounts.

NOTE: This information may be out of date due to ongoing changes to the
engine.

1. Run `setup-indica-signers clear`. This will clear all existing blockchain
   data.
2. Start all 3 nodes as described above.
3. Run `setup-indica-signers` (without `clear` argument).
4. Close all nodes and restart as described above, replacing `indica-node`
   with `indica-node-signer`.
5. Attempt to push a transaction:
```
curl --data '{"jsonrpc":"2.0","method":"personal_sendTransaction","params":[{"from":"0x002eb83d1d04ca12fe1956e67ccaa195848e437f","to":"0x00Bd138aBD70e2F00903268F3Db08f2D25677C9e","value":"0x10000"}, "richie"],"id":0}' -H "Content-Type: application/json" -X POST localhost:8500
```
6. Verify that the transaction was successfully imported to a block and synced (you may have to wait a few seconds):
```
curl --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x00Bd138aBD70e2F00903268F3Db08f2D25677C9e", "latest"],"id":1}' -H "Content-Type: application/json" -X POST localhost:8501
```
   The balance should be `0x1bc16d674ec90000`.



### Status

Experimental