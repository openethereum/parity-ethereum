// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { Address, Data, Hash, Quantity, BlockNumber, TransactionRequest, TransactionResponse } from '../types';
import { fromDecimal, withComment, Dummy } from '../helpers';

const SECTION_ACCOUNTS = 'Accounts (read-only) and Signatures';
const SECTION_DEV = 'Development';
const SECTION_MINING = 'Block Authoring (aka "mining")';
const SECTION_NET = 'Network Information';
const SECTION_NODE = 'Node Settings';
const SECTION_VAULT = 'Account Vaults';

const SUBDOC_SET = 'set';
const SUBDOC_ACCOUNTS = 'accounts';

export default {
  accountsInfo: {
    section: SECTION_ACCOUNTS,
    desc: 'Provides metadata for accounts.',
    params: [],
    returns: {
      type: Object,
      desc: 'Maps account address to metadata.',
      details: {
        name: {
          type: String,
          desc: 'Account name'
        }
      },
      example: {
        '0x0024d0c7ab4c52f723f3aaf0872b9ea4406846a4': {
          name: 'Foo'
        },
        '0x004385d8be6140e6f889833f68b51e17b6eacb29': {
          name: 'Bar'
        },
        '0x009047ed78fa2be48b62aaf095b64094c934dab0': {
          name: 'Baz'
        }
      }
    }
  },

  chainStatus: {
    section: SECTION_NET,
    desc: 'Returns the information on warp sync blocks',
    params: [],
    returns: {
      type: Object,
      desc: 'The status object',
      details: {
        blockGap: {
          type: Array,
          desc: 'Describes the gap in the blockchain, if there is one: (first, last)',
          optional: true
        }
      }
    }
  },

  changeVault: {
    section: SECTION_VAULT,
    desc: 'Changes the current valut for the account',
    params: [
      {
        type: Address,
        desc: 'Account address',
        example: '0x63Cf90D3f0410092FC0fca41846f596223979195'
      },
      {
        type: String,
        desc: 'Vault name',
        example: 'StrongVault'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'True on success',
      example: true
    }
  },

  changeVaultPassword: {
    section: SECTION_VAULT,
    desc: 'Changes the password for any given vault',
    params: [
      {
        type: String,
        desc: 'Vault name',
        example: 'StrongVault'
      },
      {
        type: String,
        desc: 'New Password',
        example: 'p@55w0rd'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'True on success',
      example: true
    }
  },

  closeVault: {
    section: SECTION_VAULT,
    desc: 'Closes a vault with the given name',
    params: [
      {
        type: String,
        desc: 'Vault name',
        example: 'StrongVault'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'True on success',
      example: true
    }
  },

  consensusCapability: {
    desc: 'Returns information on current consensus capability.',
    params: [],
    returns: {
      type: Object,
      desc: 'or `String` - Either `"capable"`, `{"capableUntil":N}`, `{"incapableSince":N}` or `"unknown"` (`N` is a block number).',
      example: 'capable'
    }
  },

  dappsPort: {
    section: SECTION_NODE,
    desc: 'Returns the port the dapps are running on, error if not enabled.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'The port number',
      example: 8080
    }
  },

  dappsInterface: {
    section: SECTION_NODE,
    desc: 'Returns the interface the dapps are running on, error if not enabled.',
    params: [],
    returns: {
      type: String,
      desc: 'The interface',
      example: '127.0.0.1'
    }
  },

  defaultAccount: {
    section: SECTION_ACCOUNTS,
    desc: 'Returns the defaultAccount that is to be used with transactions',
    params: [],
    returns: {
      type: Address,
      desc: 'The account address',
      example: '0x63Cf90D3f0410092FC0fca41846f596223979195'
    }
  },

  defaultExtraData: {
    section: SECTION_MINING,
    desc: 'Returns the default extra data',
    params: [],
    returns: {
      type: Data,
      desc: 'Extra data',
      example: '0xd5830106008650617269747986312e31342e30826c69'
    }
  },

  devLogs: {
    section: SECTION_DEV,
    desc: 'Returns latest stdout logs of your node.',
    params: [],
    returns: {
      type: Array,
      desc: 'Development logs',
      example: [
        '2017-01-20 18:14:19  Updated conversion rate to Ξ1 = US$10.63 (11199212000 wei/gas)',
        '2017-01-20 18:14:19  Configured for DevelopmentChain using InstantSeal engine',
        '2017-01-20 18:14:19  Operating mode: active',
        '2017-01-20 18:14:19  State DB configuration: fast',
        '2017-01-20 18:14:19  Starting Parity/v1.6.0-unstable-2ae8b4c-20170120/x86_64-linux-gnu/rustc1.14.0'
      ]
    }
  },

  devLogsLevels: {
    section: SECTION_DEV,
    desc: 'Returns current logging level settings. Logging level can be set with `--logging` and be one of: `""` (default), `"info"`, `"debug"`, `"warn"`, `"error"`, `"trace"`.',
    params: [],
    returns: {
      type: String,
      decs: 'Current log level.',
      example: 'debug'
    }
  },

  enode: {
    section: SECTION_NODE,
    desc: 'Returns the node enode URI.',
    params: [],
    returns: {
      type: String,
      desc: 'Enode URI',
      example: 'enode://050929adcfe47dbe0b002cb7ef2bf91ca74f77c4e0f68730e39e717f1ce38908542369ae017148bee4e0d968340885e2ad5adea4acd19c95055080a4b625df6a@172.17.0.1:30303'
    }
  },

  extraData: {
    section: SECTION_MINING,
    desc: 'Returns currently set extra data.',
    params: [],
    returns: {
      type: Data,
      desc: 'Extra data.',
      example: '0xd5830106008650617269747986312e31342e30826c69'
    }
  },

  gasFloorTarget: {
    section: SECTION_MINING,
    desc: 'Returns current target for gas floor.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Gas floor target.',
      format: 'outputBigNumberFormatter',
      example: fromDecimal(4700000)
    }
  },

  gasCeilTarget: {
    section: SECTION_MINING,
    desc: 'Returns current target for gas ceiling.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Gas ceiling target.',
      format: 'outputBigNumberFormatter',
      example: fromDecimal(6283184)
    }
  },

  gasPriceHistogram: {
    section: SECTION_NET,
    desc: 'Returns a snapshot of the historic gas prices.',
    params: [],
    returns: {
      type: Object,
      desc: 'Historic values',
      details: {
        bucketBounds: {
          type: Array,
          desc: 'Array of bound values.'
        },
        count: {
          type: Array,
          desc: 'Array of counts.'
        }
      },
      example: {
        bucketBounds: ['0x4a817c800', '0x525433d01', '0x5a26eb202', '0x61f9a2703', '0x69cc59c04', '0x719f11105', '0x7971c8606', '0x81447fb07', '0x891737008', '0x90e9ee509', '0x98bca5a0a'],
        counts: [487, 9, 7, 1, 8, 0, 0, 0, 0, 14]
      }
    }
  },

  generateSecretPhrase: {
    section: SECTION_ACCOUNTS,
    desc: 'Creates a secret phrase that can be associated with an account.',
    params: [],
    returns: {
      type: String,
      desc: 'The secret phrase.',
      example: 'boasting breeches reshape reputably exit handrail stony jargon moneywise unhinge handed ruby'
    }
  },

  getVaultMeta: {
    section: SECTION_VAULT,
    desc: 'Returns the metadata for a specific vault',
    params: [
      {
        type: String,
        desc: 'Vault name',
        example: 'StrongVault'
      }
    ],
    returns: {
      type: String,
      desc: 'The associated JSON metadata for this vault',
      example: '{"passwordHint":"something"}'
    }
  },

  hardwareAccountsInfo: {
    section: SECTION_ACCOUNTS,
    desc: 'Provides metadata for attached hardware wallets',
    params: [],
    returns: {
      type: Object,
      desc: 'Maps account address to metadata.',
      details: {
        manufacturer: {
          type: String,
          desc: 'Manufacturer'
        },
        name: {
          type: String,
          desc: 'Account name'
        }
      },
      example: {
        '0x0024d0c7ab4c52f723f3aaf0872b9ea4406846a4': {
          manufacturer: 'Ledger',
          name: 'Nano S'
        }
      }
    }
  },

  listOpenedVaults: {
    desc: 'Returns a list of all opened vaults',
    params: [],
    returns: {
      type: Array,
      desc: 'Names of all opened vaults',
      example: "['Personal']"
    }
  },

  listVaults: {
    desc: 'Returns a list of all available vaults',
    params: [],
    returns: {
      type: Array,
      desc: 'Names of all available vaults',
      example: "['Personal','Work']"
    }
  },

  localTransactions: {
    desc: 'Returns an object of current and past local transactions.',
    params: [],
    returns: {
      type: Object,
      desc: 'Mapping of transaction hashes to status objects status object.',
      example: {
        '0x09e64eb1ae32bb9ac415ce4ddb3dbad860af72d9377bb5f073c9628ab413c532': {
          status: 'mined',
          transaction: {
            from: '0x00a329c0648769a73afac7f9381e08fb43dbea72',
            to: '0x00a289b43e1e4825dbedf2a78ba60a640634dc40',
            value: '0xfffff',
            blockHash: null,
            blockNumber: null,
            creates: null,
            gas: '0xe57e0',
            gasPrice: '0x2d20cff33',
            hash: '0x09e64eb1ae32bb9ac415ce4ddb3dbad860af72d9377bb5f073c9628ab413c532',
            input: '0x',
            minBlock: null,
            networkId: null,
            nonce: '0x0',
            publicKey: '0x3fa8c08c65a83f6b4ea3e04e1cc70cbe3cd391499e3e05ab7dedf28aff9afc538200ff93e3f2b2cb5029f03c7ebee820d63a4c5a9541c83acebe293f54cacf0e',
            raw: '0xf868808502d20cff33830e57e09400a289b43e1e4825dbedf2a78ba60a640634dc40830fffff801ca034c333b0b91cd832a3414d628e3fea29a00055cebf5ba59f7038c188404c0cf3a0524fd9b35be170439b5ffe89694ae0cfc553cb49d1d8b643239e353351531532',
            standardV: '0x1',
            v: '0x1c',
            r: '0x34c333b0b91cd832a3414d628e3fea29a00055cebf5ba59f7038c188404c0cf3',
            s: '0x524fd9b35be170439b5ffe89694ae0cfc553cb49d1d8b643239e353351531532',
            transactionIndex: null
          }
        },
        '0x...': new Dummy('{ ... }')
      }
    }
  },

  minGasPrice: {
    section: SECTION_MINING,
    desc: 'Returns currently set minimal gas price',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Minimal Gas Price',
      format: 'outputBigNumberFormatter',
      example: fromDecimal(11262783488)
    }
  },

  mode: {
    section: SECTION_NODE,
    desc: 'Get the mode. Results one of: `"active"`, `"passive"`, `"dark"`, `"offline"`.',
    params: [],
    returns: {
      type: String,
      desc: 'The mode.',
      example: 'active'
    }
  },

  netChain: {
    section: SECTION_NET,
    desc: 'Returns the name of the connected chain.',
    params: [],
    returns: {
      type: String,
      desc: 'chain name.',
      example: 'homestead'
    }
  },

  netPeers: {
    section: SECTION_NET,
    desc: 'Returns number of peers.',
    params: [],
    returns: {
      type: Object,
      desc: 'Number of peers',
      details: {
        active: {
          type: Quantity,
          desc: 'Number of active peers.'
        },
        connected: {
          type: Quantity,
          desc: 'Number of connected peers.'
        },
        max: {
          type: Quantity,
          desc: 'Maximum number of connected peers.'
        },
        peers: {
          type: Array,
          desc: 'List of all peers with details.'
        }
      },
      example: {
        active: 0,
        connected: 25,
        max: 25,
        peers: [new Dummy('{ ... }, { ... }, { ... }, ...')]
      }
    }
  },

  netPort: {
    section: SECTION_NET,
    desc: 'Returns network port the node is listening on.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Port number',
      example: 30303
    }
  },

  newVault: {
    section: SECTION_VAULT,
    desc: 'Creates a new vault with the given name & password',
    params: [
      {
        type: String,
        desc: 'Vault name',
        example: 'StrongVault'
      },
      {
        type: String,
        desc: 'Password',
        example: 'p@55w0rd'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'True on success',
      example: true
    }
  },

  nextNonce: {
    section: SECTION_NET,
    desc: 'Returns next available nonce for transaction from given account. Includes pending block and transaction queue.',
    params: [
      {
        type: Address,
        desc: 'Account',
        example: '0x00A289B43e1e4825DbEDF2a78ba60a640634DC40'
      }
    ],
    returns: {
      type: Quantity,
      desc: 'Next valid nonce',
      example: fromDecimal(12)
    }
  },

  nodeName: {
    section: SECTION_NODE,
    desc: 'Returns node name, set when starting parity with `--identity NAME`.',
    params: [],
    returns: {
      type: String,
      desc: 'Node name.',
      example: 'Doge'
    }
  },

  openVault: {
    section: SECTION_VAULT,
    desc: 'Opens a vault with the given name & password',
    params: [
      {
        type: String,
        desc: 'Vault name',
        example: 'StrongVault'
      },
      {
        type: String,
        desc: 'Password',
        example: 'p@55w0rd'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'True on success',
      example: true
    }
  },

  pendingTransactions: {
    section: SECTION_NET,
    desc: 'Returns a list of transactions currently in the queue.',
    params: [],
    returns: {
      type: Array,
      desc: 'Transactions ordered by priority',
      details: TransactionResponse.details,
      example: [
        {
          blockHash: null,
          blockNumber: null,
          creates: null,
          from: '0xee3ea02840129123d5397f91be0391283a25bc7d',
          gas: '0x23b58',
          gasPrice: '0xba43b7400',
          hash: '0x160b3c30ab1cf5871083f97ee1cee3901cfba3b0a2258eb337dd20a7e816b36e',
          input: '0x095ea7b3000000000000000000000000bf4ed7b27f1d666546e30d74d50d173d20bca75400000000000000000000000000002643c948210b4bd99244ccd64d5555555555',
          minBlock: null,
          networkId: 1,
          nonce: '0x5',
          publicKey: '0x96157302dade55a1178581333e57d60ffe6fdf5a99607890456a578b4e6b60e335037d61ed58aa4180f9fd747dc50d44a7924aa026acbfb988b5062b629d6c36',
          r: '0x92e8beb19af2bad0511d516a86e77fa73004c0811b2173657a55797bdf8558e1',
          raw: '0xf8aa05850ba43b740083023b5894bb9bc244d798123fde783fcc1c72d3bb8c18941380b844095ea7b3000000000000000000000000bf4ed7b27f1d666546e30d74d50d173d20bca75400000000000000000000000000002643c948210b4bd99244ccd64d555555555526a092e8beb19af2bad0511d516a86e77fa73004c0811b2173657a55797bdf8558e1a062b4d4d125bbcb9c162453bc36ca156537543bb4414d59d1805d37fb63b351b8',
          s: '0x62b4d4d125bbcb9c162453bc36ca156537543bb4414d59d1805d37fb63b351b8',
          standardV: '0x1',
          to: '0xbb9bc244d798123fde783fcc1c72d3bb8c189413',
          transactionIndex: null,
          v: '0x26',
          value: '0x0'
        },
        new Dummy('{ ... }'),
        new Dummy('{ ... }')
      ]
    }
  },

  pendingTransactionsStats: {
    desc: 'Returns propagation stats for transactions in the queue.',
    params: [],
    returns: {
      type: Object,
      desc: 'mapping of transaction hashes to stats.',
      example: {
        '0xdff37270050bcfba242116c745885ce2656094b2d3a0f855649b4a0ee9b5d15a': {
          firstSeen: 3032066,
          propagatedTo: {
            '0x605e04a43b1156966b3a3b66b980c87b7f18522f7f712035f84576016be909a2798a438b2b17b1a8c58db314d88539a77419ca4be36148c086900fba487c9d39': 1,
            '0xbab827781c852ecf52e7c8bf89b806756329f8cbf8d3d011e744a0bc5e3a0b0e1095257af854f3a8415ebe71af11b0c537f8ba797b25972f519e75339d6d1864': 1
          }
        }
      }
    }
  },

  phraseToAddress: {
    section: SECTION_ACCOUNTS,
    desc: 'Converts a secret phrase into the corresponding address.',
    params: [
      {
        type: String,
        desc: 'The phrase',
        example: 'stylus outing overhand dime radial seducing harmless uselessly evasive tastiness eradicate imperfect'
      }
    ],
    returns: {
      type: Address,
      desc: 'Corresponding address',
      example: '0x004385d8be6140e6f889833f68b51e17b6eacb29'
    }
  },

  releasesInfo: {
    desc: 'returns a ReleasesInfo object describing the current status of releases',
    params: [],
    returns: {
      type: Object,
      desc: 'Information on current releases, `null` if not available.',
      details: {
        fork: {
          type: Quantity,
          desc: 'Block number representing the last known fork for this chain, which may be in the future.'
        },
        minor: {
          type: Object,
          desc: 'Information about latest minor update to current version, `null` if this is the latest minor version.'
        },
        track: {
          type: Object,
          desc: 'Information about the latest release in this track.'
        }
      },
      example: null
    }
  },

  registryAddress: {
    section: SECTION_NET,
    desc: 'The address for the global registry.',
    params: [],
    returns: {
      type: Address,
      desc: 'The registry address.',
      example: '0x3bb2bb5c6c9c9b7f4ef430b47dc7e026310042ea'
    }
  },

  rpcSettings: {
    section: SECTION_NET,
    desc: 'Provides current JSON-RPC API settings.',
    params: [],
    returns: {
      type: Object,
      desc: 'JSON-RPC settings.',
      details: {
        enabled: {
          type: Boolean,
          desc: '`true` if JSON-RPC is enabled (default).'
        },
        interface: {
          type: String,
          desc: 'Interface on which JSON-RPC is running.'
        },
        port: {
          type: Quantity,
          desc: 'Port on which JSON-RPC is running.'
        }
      },
      example: {
        enabled: true,
        interface: 'local',
        port: 8545
      }
    }
  },

  setVaultMeta: {
    section: SECTION_VAULT,
    desc: 'Sets the metadata for a specific vault',
    params: [
      {
        type: String,
        desc: 'Vault name',
        example: 'StrongVault'
      },
      {
        type: String,
        desc: 'The metadata as a JSON string',
        example: '{"passwordHint":"something"}'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'The boolean call result, true on success',
      example: true
    }
  },

  signerPort: {
    section: SECTION_NODE,
    desc: 'Returns the port the signer is running on, error if not enabled',
    params: [],
    returns: {
      type: Quantity,
      desc: 'The port number',
      example: 8180
    }
  },

  transactionsLimit: {
    section: SECTION_MINING,
    desc: 'Changes limit for transactions in queue.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Current max number of transactions in queue.',
      format: 'outputBigNumberFormatter',
      example: 1024
    }
  },

  unsignedTransactionsCount: {
    section: SECTION_NET,
    desc: 'Returns number of unsigned transactions when running with Trusted Signer. Error otherwise',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Number of unsigned transactions',
      example: 0
    }
  },

  versionInfo: {
    desc: 'Provides information about running version of Parity.',
    params: [],
    returns: {
      type: Object,
      desc: 'Information on current version.',
      details: {
        hash: {
          type: Hash,
          desc: '20 Byte hash of the current build.'
        },
        track: {
          type: String,
          desc: 'Track on which it was released, one of: `"stable"`, `"beta"`, `"nightly"`, `"testing"`, `"null"` (unknown or self-built).'
        },
        version: {
          type: Object,
          desc: 'Version number composed of `major`, `minor` and `patch` integers.'
        }
      },
      example: {
        hash: '0x2ae8b4ca278dd7b896090366615fef81cbbbc0e0',
        track: 'null',
        version: {
          major: 1,
          minor: 6,
          patch: 0
        }
      }
    }
  },

  listAccounts: {
    desc: 'Returns all addresses if Fat DB is enabled (`--fat-db`), `null` otherwise.',
    section: SECTION_ACCOUNTS,
    params: [
      {
        type: Quantity,
        desc: 'Integer number of addresses to display in a batch.',
        example: 5
      },
      {
        type: Address,
        desc: '20 Bytes - Offset address from which the batch should start in order, or `null`.',
        example: null
      },
      {
        type: BlockNumber,
        desc: 'integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`.',
        format: 'inputDefaultBlockNumberFormatter',
        optional: true
      }
    ],
    returns: {
      type: Array,
      desc: 'Requested number of `Address`es or `null` if Fat DB is not enabled.',
      example: [
        '0x7205b1bb42edce6e0ced37d1fd0a9d684f5a860f',
        '0x98a2559a814c300b274325c92df1682ae0d344e3',
        '0x2d7a7d0adf9c5f9073fefbdc18188bd23c68b633',
        '0xd4bb3284201db8b03c06d8a3057dd32538e3dfda',
        '0xa6396904b08aa31300ca54278b8e066ecc38e4a0'
      ]
    }
  },

  listStorageKeys: {
    desc: 'Returns all storage keys of the given address (first parameter) if Fat DB is enabled (`--fat-db`), `null` otherwise.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - Account for which to retrieve the storage keys.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: Quantity,
        desc: 'Integer number of addresses to display in a batch.',
        example: 5
      },
      {
        type: Hash,
        desc: '32 Bytes - Offset storage key from which the batch should start in order, or `null`.',
        example: null
      },
      {
        type: BlockNumber,
        desc: 'integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`.',
        format: 'inputDefaultBlockNumberFormatter',
        optional: true
      }
    ],
    returns: {
      type: Array,
      desc: 'Requested number of 32 byte long storage keys for the given account or `null` if Fat DB is not enabled.',
      example: [
        '0xaab1a2940583e213f1d57a3ed358d5f5406177c8ff3c94516bfef3ea62d00c22',
        '0xba8469eca5641b186e86cbc5343dfa5352df04feb4564cd3cf784f213aaa0319',
        '0x769d107ba778d90205d7a159e820c41c20bf0783927b426c602561e74b7060e5',
        '0x0289865bcaa58f7f5bf875495ac7af81e3630eb88a3a0358407c7051a850624a',
        '0x32e0536502b9163b0a1ce6e3aabd95fa4a2bf602bbde1b9118015648a7a51178'
      ]
    }
  },

  encryptMessage: {
    desc: 'Encrypt some data with a public key under ECIES.',
    params: [
      {
        type: Hash,
        desc: 'Public EC key generated with `secp256k1` curve, truncated to the last 64 bytes.',
        example: '0xD219959D466D666060284733A80DDF025529FEAA8337169540B3267B8763652A13D878C40830DD0952639A65986DBEC611CF2171A03CFDC37F5A40537068AA4F'
      },
      {
        type: Data,
        desc: 'The message to encrypt.',
        example: withComment('0x68656c6c6f20776f726c64', '"hello world"')
      }
    ],
    returns: {
      type: Data,
      desc: 'Encrypted message.',
      example: '0x0491debeec5e874a453f84114c084c810708ebcb553b02f1b8c05511fa4d1a25fa38eb49a32c815e2b39b7bcd56d66648bf401067f15413dae683084ca7b01e21df89be9ec4bc6c762a657dbd3ba1540f557e366681b53629bb2c02e1443b5c0adc6b68f3442c879456d6a21ec9ed07847fa3c3ecb73ec7ee9f8e32d'
    }
  },

  futureTransactions: {
    desc: 'Returns all future transactions from transaction queue.',
    params: [],
    returns: {
      type: Array,
      desc: 'Transaction list.',
      details: TransactionResponse.details,
      example: [
        {
          hash: '0x80de421cd2e7e46824a91c343ca42b2ff339409eef09e2d9d73882462f8fce31',
          nonce: '0x1',
          blockHash: null,
          blockNumber: null,
          transactionIndex: null,
          from: '0xe53e478c072265e2d9a99a4301346700c5fbb406',
          to: '0xf5d405530dabfbd0c1cab7a5812f008aa5559adf',
          value: '0x2efc004ac03a4996',
          gasPrice: '0x4a817c800',
          gas: '0x5208',
          input: '0x',
          creates: null,
          raw: '0xf86c018504a817c80082520894f5d405530dabfbd0c1cab7a5812f008aa5559adf882efc004ac03a49968025a0b40c6967a7e8bbdfd99a25fd306b9ef23b80e719514aeb7ddd19e2303d6fc139a06bf770ab08119e67dc29817e1412a0e3086f43da308c314db1b3bca9fb6d32bd',
          publicKey: '0xeba33fd74f06236e17475bc5b6d1bac718eac048350d77d3fc8fbcbd85782a57c821255623c4fd1ebc9d555d07df453b2579ee557b7203fc256ca3b3401e4027',
          networkId: 1,
          standardV: '0x0',
          v: '0x25',
          r: '0xb40c6967a7e8bbdfd99a25fd306b9ef23b80e719514aeb7ddd19e2303d6fc139',
          s: '0x6bf770ab08119e67dc29817e1412a0e3086f43da308c314db1b3bca9fb6d32bd',
          minBlock: null
        },
        new Dummy('{ ... }, { ... }, ...')
      ]
    }
  },

  /*
   * `parity_accounts` module methods
   * ================================
   */
  allAccountsInfo: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'returns a map of accounts as an object.',
    params: [],
    returns: {
      type: Array,
      desc: 'Account metadata.',
      details: {
        name: {
          type: String,
          desc: 'Account name.'
        },
        meta: {
          type: String,
          desc: 'Encoded JSON string the defines additional account metadata.'
        },
        uuid: {
          type: String,
          desc: 'The account Uuid, or `null` if not available/unknown/not applicable.'
        }
      },
      example: {
        '0x00a289b43e1e4825dbedf2a78ba60a640634dc40': {
          meta: '{}',
          name: 'Foobar',
          uuid: '0b9e70e6-235b-682d-a15c-2a98c71b3945'
        }
      }
    }
  },

  newAccountFromPhrase: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Creates a new account from a recovery phrase.',
    params: [
      {
        type: String,
        desc: 'Recovery phrase.',
        example: 'stylus outing overhand dime radial seducing harmless uselessly evasive tastiness eradicate imperfect'
      },
      {
        type: String,
        desc: 'Password.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Address,
      desc: 'The created address.',
      example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
    }
  },

  newAccountFromSecret: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Creates a new account from a private ethstore secret key.',
    params: [
      {
        type: Data,
        desc: 'Secret, 32-byte hex',
        example: '0x1db2c0cf57505d0f4a3d589414f0a0025ca97421d2cd596a9486bc7e2cd2bf8b'
      },
      {
        type: String,
        desc: 'Password',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Address,
      desc: 'The created address.',
      example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
    }
  },

  newAccountFromWallet: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Creates a new account from a JSON import',
    params: [
      {
        type: String,
        desc: 'Wallet JSON encoded to a string.',
        example: '{"id": "9c62e86b-3cf9...", ...}'
      },
      {
        type: String,
        desc: 'Password.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Address,
      desc: 'The created address',
      example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
    }
  },

  setAccountName: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Sets a name for the account',
    params: [
      {
        type: Address,
        desc: 'Address',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: String,
        desc: 'Name',
        example: 'Foobar'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  setAccountMeta: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Sets metadata for the account',
    params: [
      {
        type: Address,
        desc: 'Address',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: String,
        desc: 'Metadata (JSON encoded)',
        example: '{"foo":"bar"}'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  testPassword: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Checks if a given password can unlock a given account, without actually unlocking it.',
    params: [
      {
        type: Address,
        desc: 'Account to test.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: String,
        desc: 'Password to test.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the account and password are valid.',
      example: true
    }
  },

  changePassword: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Change the password for a given account.',
    params: [
      {
        type: Address,
        desc: 'Address of the account.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: String,
        desc: 'Old password.',
        example: 'hunter2'
      },
      {
        type: String,
        desc: 'New password.',
        example: 'bazqux5'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  killAccount: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Deletes an account.',
    params: [
      {
        type: Address,
        desc: 'The account to remove.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: String,
        desc: 'Account password.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  removeAddress: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Removes an address from the addressbook.',
    params: [
      {
        type: Address,
        desc: 'The address to remove.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true`if the call was successful.',
      example: true
    }
  },

  setDappAddresses: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Sets the available addresses for a dapp. When provided with non-empty list changes the default account as well.',
    params: [
      {
        type: String,
        desc: 'Dapp Id.',
        example: 'web'
      },
      {
        type: Array,
        desc: 'Array of available accounts available to the dapp or `null` for default list.',
        example: ['0x407d73d8a49eeb85d32cf465507dd71d507100c1']
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  getDappAddresses: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Returns the list of accounts available to a specific dapp.',
    params: [
      {
        type: String,
        desc: 'Dapp Id.',
        example: 'web'
      }
    ],
    returns: {
      type: Array,
      desc: 'The list of available accounts.',
      example: ['0x407d73d8a49eeb85d32cf465507dd71d507100c1']
    }
  },

  setDappDefaultAddress: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Changes dapp default address. Does not affect other accounts exposed for this dapp, but default account will always be retured as the first one.',
    params: [
      {
        type: String,
        desc: 'Dapp Id.',
        example: 'web'
      },
      {
        type: Address,
        desc: 'Default Address.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful',
      example: true
    }
  },

  getDappDefaultAddress: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Returns a default account available to a specific dapp.',
    params: [
      {
        type: String,
        desc: 'Dapp Id.',
        example: 'web'
      }
    ],
    returns: {
      type: Address,
      desc: 'Default Address',
      example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
    }
  },

  setNewDappsAddresses: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Sets the list of accounts available to new dapps.',
    params: [
      {
        type: Array,
        desc: 'List of accounts available by default or `null` for all accounts.',
        example: ['0x407d73d8a49eeb85d32cf465507dd71d507100c1']
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful',
      example: true
    }
  },

  getNewDappsAddresses: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Returns the list of accounts available to a new dapps.',
    params: [],
    returns: {
      type: Array,
      desc: 'The list of available accounts, can be `null`.',
      example: ['0x407d73d8a49eeb85d32cf465507dd71d507100c1']
    }
  },

  setNewDappsDefaultAddress: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Changes global default address. This setting may be overriden for a specific dapp.',
    params: [
      {
        type: Address,
        desc: 'Default Address.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful',
      example: true
    }
  },

  getNewDappsDefaultAddress: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Returns a default account available to dapps.',
    params: [],
    returns: {
      type: Address,
      desc: 'Default Address',
      example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
    }
  },

  listRecentDapps: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Returns a list of the most recent active dapps.',
    params: [],
    returns: {
      type: Array,
      desc: 'Array of Dapp Ids.',
      example: ['web']
    }
  },

  importGethAccounts: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Imports a list of accounts from Geth.',
    params: [
      {
        type: Array,
        desc: 'List of the Geth addresses to import.'
      }
    ],
    returns: {
      type: Array,
      desc: 'Array of the imported addresses.'
    }
  },

  listGethAccounts: {
    subdoc: SUBDOC_ACCOUNTS,
    desc: 'Returns a list of the accounts available from Geth.',
    params: [],
    returns: {
      type: Array,
      desc: '20 Bytes addresses owned by the client.'
    }
  },

  /*
   * `parity_set` module methods
   * ===========================
   */
  setMinGasPrice: {
    subdoc: SUBDOC_SET,
    desc: 'Changes minimal gas price for transaction to be accepted to the queue.',
    params: [
      {
        type: Quantity,
        desc: 'Minimal gas price',
        format: 'utils.toHex',
        example: fromDecimal(1000)
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful',
      example: true
    }
  },

  setGasFloorTarget: {
    subdoc: SUBDOC_SET,
    desc: 'Sets a new gas floor target for mined blocks..',
    params: [
      {
        type: Quantity,
        desc: '(default: `0x0`) Gas floor target.',
        format: 'utils.toHex',
        example: fromDecimal(1000)
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  setGasCeilTarget: {
    subdoc: SUBDOC_SET,
    desc: 'Sets new gas ceiling target for mined blocks.',
    params: [
      {
        type: Quantity,
        desc: '(default: `0x0`) Gas ceiling target.',
        format: 'utils.toHex',
        example: fromDecimal(10000000000)
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  setExtraData: {
    subdoc: SUBDOC_SET,
    desc: 'Changes extra data for newly mined blocks',
    params: [
      {
        type: Data,
        desc: 'Extra Data',
        format: 'utils.toHex',
        example: '0x'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful',
      example: true
    }
  },

  setAuthor: {
    subdoc: SUBDOC_SET,
    desc: 'Changes author (coinbase) for mined blocks.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - Address',
        format: 'inputAddressFormatter',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  setMaxTransactionGas: {
    subdoc: SUBDOC_SET,
    desc: 'Sets the maximum amount of gas a single transaction may consume.',
    params: [
      {
        type: Quantity,
        desc: 'Gas amount',
        format: 'utils.toHex',
        example: fromDecimal(100000)
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call was successful.',
      example: true
    }
  },

  setTransactionsLimit: {
    subdoc: SUBDOC_SET,
    desc: 'Changes limit for transactions in queue.',
    params: [
      {
        type: Quantity,
        desc: 'New Limit',
        format: 'utils.toHex',
        example: fromDecimal(1000)
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful',
      example: true
    }
  },

  addReservedPeer: {
    subdoc: SUBDOC_SET,
    desc: 'Add a reserved peer.',
    params: [
      {
        type: String,
        desc: 'Enode address',
        example: 'enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if successful.',
      example: true
    }
  },

  removeReservedPeer: {
    subdoc: SUBDOC_SET,
    desc: 'Remove a reserved peer.',
    params: [
      {
        type: String,
        desc: 'Encode address',
        example: 'enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@22.99.55.44:7770'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if successful.',
      example: true
    }
  },

  dropNonReservedPeers: {
    subdoc: SUBDOC_SET,
    desc: 'Set Parity to drop all non-reserved peers. To restore default behavior call [parity_acceptNonReservedPeers](#parity_acceptnonreservedpeers).',
    params: [],
    returns: {
      type: Boolean,
      desc: '`true` if successful.',
      example: true
    }
  },

  acceptNonReservedPeers: {
    subdoc: SUBDOC_SET,
    desc: 'Set Parity to accept non-reserved peers (default behavior).',
    params: [],
    returns: {
      type: Boolean,
      desc: '`true` if successful.',
      example: true
    }
  },

  hashContent: {
    subdoc: SUBDOC_SET,
    desc: 'Creates a hash of a file at a given URL.',
    params: [
      {
        type: String,
        desc: 'The url of the content.',
        example: 'https://raw.githubusercontent.com/ethcore/parity/master/README.md'
      }
    ],
    returns: {
      type: Hash,
      desc: 'The SHA-3 hash of the content.',
      example: '0x2547ea3382099c7c76d33dd468063b32d41016aacb02cbd51ebc14ff5d2b6a43'
    }
  },

  setMode: {
    subdoc: SUBDOC_SET,
    desc: 'Changes the operating mode of Parity.',
    params: [
      {
        type: String,
        desc: 'The mode to set, one of:\n  * `"active"` - Parity continuously syncs the chain.\n  * `"passive"` - Parity syncs initially, then sleeps and wakes regularly to resync.\n  * `"dark"` - Parity syncs only when the RPC is active.\n  * `"offline"` - Parity doesn\'t sync.\n',
        example: 'passive'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the call succeeded.',
      example: true
    }
  },

  setEngineSigner: {
    subdoc: SUBDOC_SET,
    desc: 'Sets an authority account for signing consensus messages. For more information check the [[Proof of Authority Chains]] page.',
    params: [
      {
        type: Address,
        desc: 'Identifier of a valid authority account.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: String,
        desc: 'Passphrase to unlock the account.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'True if the call succeeded',
      example: true
    }
  },

  upgradeReady: {
    subdoc: SUBDOC_SET,
    desc: 'Returns a ReleaseInfo object describing the release which is available for upgrade or `null` if none is available.',
    params: [],
    returns: {
      type: Object,
      desc: 'Details or `null` if no new release is available.',
      details: {
        version: {
          type: Object,
          desc: 'Information on the version.'
        },
        is_critical: {
          type: Boolean,
          desc: 'Does this release contain critical security updates?'
        },
        fork: {
          type: Quantity,
          desc: 'The latest fork that this release can handle.'
        },
        binary: {
          type: Data,
          desc: 'Keccak-256 checksum of the release parity binary, if known.',
          optional: true
        }
      },
      example: null
    }
  },

  executeUpgrade: {
    subdoc: SUBDOC_SET,
    desc: 'Attempts to upgrade Parity to the version specified in [parity_upgradeReady](#parity_upgradeready).',
    params: [],
    returns: {
      type: Boolean,
      desc: 'returns `true` if the upgrade to the new release was successfully executed, `false` if not.',
      example: true
    }
  },

  /*
   * `parity_signing` trait methods (rolled into `parity` module)
   * ============================================================
   */
  postSign: {
    section: SECTION_ACCOUNTS,
    desc: 'Request an arbitrary transaction to be signed by an account.',
    params: [
      {
        type: Address,
        desc: 'Account address.',
        example: '0xb60e8dd61c5d32be8058bb8eb970870f07233155'
      },
      {
        type: Hash,
        desc: 'Transaction hash.',
        example: '0x8cda01991ae267a539135736132f1f987e76868ce0269b7537d3aab37b7b185e'
      }
    ],
    returns: {
      type: Quantity,
      desc: 'The id of the request to the signer. If the account was already unlocked, returns `Hash` of the transaction instead.',
      example: '0x1'
    }
  },

  postTransaction: {
    section: SECTION_ACCOUNTS,
    desc: 'Posts a transaction to the signer without waiting for the signer response.',
    params: [
      {
        type: TransactionRequest,
        desc: 'see [`eth_sendTransaction`](JSONRPC-eth-module#eth_sendtransaction).',
        format: 'inputCallFormatter',
        example: {
          from: '0xb60e8dd61c5d32be8058bb8eb970870f07233155',
          to: '0xd46e8dd67c5d32be8058bb8eb970870f07244567',
          value: fromDecimal(2441406250)
        }
      }
    ],
    returns: {
      type: Quantity,
      desc: 'The id of the request to the signer. If the account was already unlocked, returns `Hash` of the transaction instead.',
      format: 'utils.toDecimal',
      example: '0x1'
    }
  },

  checkRequest: {
    section: SECTION_ACCOUNTS,
    desc: 'Get the the transaction hash of the request previously posted to [`parity_postTransaction`](#parity_posttransaction) or [`parity_postSign`](#parity_postsign). Will return a JSON-RPC error if the request was rejected.',
    params: [
      {
        type: Quantity,
        desc: 'The id of the request sent to the signer.',
        example: '0x1'
      }
    ],
    returns: {
      type: Hash,
      desc: '32 Bytes - the transaction hash or `null` if the request hasn\'t been signed yet.',
      example: '0xde8dfd9642f7eeef12402f2a560dbf40921b4f0bda01fb84709b9d71f6c181be'
    }
  },

  decryptMessage: {
    desc: 'Decrypt a message encrypted with a ECIES public key.',
    params: [
      {
        type: Address,
        desc: 'Account which can decrypt the message.',
        example: '0x00a329c0648769a73afac7f9381e08fb43dbea72'
      },
      {
        type: Data,
        desc: 'Encrypted message.',
        example: '0x0405afee7fa2ab3e48c27b00d543389270cb7267fc191ca1311f297255a83cbe8d77a4ba135b51560700a582924fa86d2b19029fcb50d2b68d60a7df1ba81df317a19c8def117f2b9cf8c2618be0e3f146a5272fb9e5528719d2d7a1bd91fa620901cffa756305c79c093e7af30fa3c1587029421351c34a7c1e5a2b'
      }
    ],
    returns: {
      type: Data,
      desc: 'Decrypted message.',
      example: withComment('0x68656c6c6f20776f726c64', 'hello world')
    }
  }
};
