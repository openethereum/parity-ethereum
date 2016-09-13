// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

export default function (rpc) {
  return {
    getCoins: function () {
      return rpc.get('getcoins');
    },

    getMarketInfo: function (pair) {
      return rpc.get(`marketinfo/${pair}`);
    },

    getStatus: function (depositAddress) {
      return rpc.get(`txStat/${depositAddress}`);
    },

    shift: function (toAddress, returnAddress, pair) {
      return rpc.post('shift', {
        withdrawal: toAddress,
        pair: pair,
        returnAddress: returnAddress
      });
    }
  };
}
