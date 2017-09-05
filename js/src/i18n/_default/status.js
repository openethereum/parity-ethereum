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

export default {
  debug: {
    reverse: `Reverse Order`,
    stopped: `Refresh and display of logs from Parity is currently stopped via the UI, start it to see the latest updates.`,
    title: `Node Logs`
  },
  health: {
    no: `no`,
    peers: `Connected Peers`,
    sync: `Chain Synchronized`,
    time: `Time Synchronized`,
    title: `Node Health`,
    yes: `yes`
  },
  miningSettings: {
    input: {
      author: {
        hint: `the mining author`,
        label: `author`
      },
      extradata: {
        hint: `extra data for mined blocks`,
        label: `extradata`
      },
      gasFloor: {
        hint: `the gas floor target for mining`,
        label: `gas floor target`
      },
      gasPrice: {
        hint: `the minimum gas price for mining`,
        label: `minimal gas price`
      }
    },
    title: `mining settings`
  },
  peers: {
    table: {
      header: {
        caps: `Capabilities`,
        ethDiff: `Difficulty (ETH)`,
        ethHeader: `Header (ETH)`,
        id: `ID`,
        name: `Name`,
        remoteAddress: `Remote Address`
      }
    },
    title: `network peers`
  },
  status: {
    hashrate: `{hashrate} H/s`,
    input: {
      chain: `chain`,
      enode: `enode`,
      no: `no`,
      port: `network port`,
      rpcEnabled: `rpc enabled`,
      rpcInterface: `rpc interface`,
      rpcPort: `rpc port`,
      yes: `yes`
    },
    title: {
      bestBlock: `best block`,
      hashRate: `hash rate`,
      network: `network settings`,
      node: `Node`,
      peers: `peers`
    }
  },
  title: `Status`
};
