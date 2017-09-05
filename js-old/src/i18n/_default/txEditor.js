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
  condition: {
    block: {
      hint: `The minimum block to send from`,
      label: `Transaction send block`
    },
    blocknumber: `Send after BlockNumber`,
    date: {
      hint: `The minimum date to send from`,
      label: `Transaction send date`
    },
    datetime: `Send after Date & Time`,
    label: `Condition where transaction activates`,
    none: `No conditions`,
    time: {
      hint: `The minimum time to send from`,
      label: `Transaction send time`
    }
  },
  gas: {
    info: `You can choose the gas price based on the distribution of recent included transaction gas prices. The lower the gas price is, the cheaper the transaction will be. The higher the gas price is, the faster it should get mined by the network.`
  }
};
