## OpenEthereum v3.1RC1

OpenEthereum 3.1rc1 is a candidate release based on v2.5.13 which is the last stable version known of the client that does not include any of the issues introduced in v2.7. 
It removes non core features like Ethereum Classic, Private Transactions, Light Client, Updater, IPFS and Swarm support, currently deprecated flags such as expanse, kotti, mordor testnets.

Database migration utility currently in beta: https://github.com/openethereum/3.1-db-upgrade-tool

The full list of included changes from v2.5.13 to v3.1:

- Remove classic, kotti, mordor, expanse (#52) 
- Added bad block header hash for ropsten (#49) 
- Remove accounts bloom (#33) 
- Bump jsonrpc-- to v15 
- Implement eth/64, remove eth/62 (#46) 
- No snapshotting by default (#11814) 
- Update Ellaism chainspec 
- Prometheus, heavy memory calls removed (#27) 
- Update ethereum/tests 
- Implement JSON test suite (#11801) 
- Fix issues during block sync (#11265) 
- Fix race same block (#11400) 
- EIP-2537: Precompile for BLS12-381 curve operations (#11707) 
- Remove private transactions 
- Remove GetNodeData 
- Remove IPFS integration (#11532) 
- Remove updater 
- Remove light client 
- Remove C and Java bindings (#11346) 
- Remove whisper (#10855) 
- EIP-2315: Simple Subroutines for the EVM (#11629) 
- Remove deprecated flags (removal of --geth flag)
- Remove support for hardware wallets (#10678) 
- Update bootnodes 

