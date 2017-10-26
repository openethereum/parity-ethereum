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
    reverse: `Omgekeerde volgorde`,
    stopped: `De live weergave van de Parity logboeken is momenteel gestopt via de UI, start de live weergave om de laatste updates te zien.`,
    title: `Node Logboeken`
  },
  miningSettings: {
    input: {
      author: {
        hint: `de mining auteur`,
        label: `auteur`
      },
      extradata: {
        hint: `extra data voor mined blokken`,
        label: `extra data`
      },
      gasFloor: {
        hint: `het gas-floor doel voor mining`,
        label: `gas-floor doel`
      },
      gasPrice: {
        hint: `de minimale gas prijs voor mining`,
        label: `minimale gas prijs`
      }
    },
    title: `mining instellingen`
  },
  status: {
    hashrate: `{hashrate} H/s`,
    input: {
      chain: `chain`,
      enode: `enode`,
      no: `nee`,
      peers: `peers`,
      port: `netwerk poort`,
      rpcEnabled: `rpc ingeschakeld`,
      rpcInterface: `rpc interface`,
      rpcPort: `rpc poort`,
      yes: `ja`
    },
    title: {
      bestBlock: `beste block`,
      hashRate: `hash rate`,
      network: `netwerk instellingen`,
      node: `Node`,
      peers: `peers`
    }
  },
  title: `Status`
};
