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

export default function web3extensions (web3) {
  const { Method, formatters } = web3._extend;

  return [{
    property: 'personal',
    methods: [
      new Method({
        name: 'signAndSendTransaction',
        call: 'personal_signAndSendTransaction',
        params: 2,
        inputFormatter: [formatters.inputTransactionFormatter, null]
      }),
      new Method({
        name: 'signerEnabled',
        call: 'personal_signerEnabled',
        params: 0,
        inputFormatter: []
      })
    ],
    properties: []
  }, {
    property: 'ethcore',
    methods: [
      new Method({
        name: 'getNetPeers',
        call: 'ethcore_netPeers',
        params: 0,
        outputFormatter: x => x
      }),
      new Method({
        name: 'getNetChain',
        call: 'ethcore_netChain',
        params: 0,
        outputFormatter: x => x
      }),
      new Method({
        name: 'gasPriceStatistics',
        call: 'ethcore_gasPriceStatistics',
        params: 0,
        outputFormatter: a => a.map(web3.toBigNumber)
      }),
      new Method({
        name: 'unsignedTransactionsCount',
        call: 'ethcore_unsignedTransactionsCount',
        params: 0,
        inputFormatter: []
      })
    ],
    properties: []
  }];
}
