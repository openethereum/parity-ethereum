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
      hint: `在某个区块高度后发送`, // The minimum block to send from
      label: `交易发送区块`// Transaction send block
    },
    blocknumber: `在某个区块后发送`, // Send after BlockNumber
    date: {
      hint: `在某日后发送`, // The minimum date to send from
      label: `交易发送日期`// Transaction send date
    },
    datetime: `在某日某时后发送`, // Send after Date & Tim
    label: `交易激活的条件`, // Condition where transaction activates
    none: `无条件`, // No conditions
    time: {
      hint: `在某时间后发送`, // The minimum time to send from
      label: `交易发送时间`// Transaction send time
    }
  },
  gas: {
    info: `你可以基于最近的交易gas价格的分布选择gas价格。 gas价格越低，交易费用越便宜。 gas 价格越高，交易被网络打包的速度越快。`
    // You can choose the gas price based on the distribution of recent included transaction gas prices.The lower the gas price is, the cheaper the transaction will be.The higher the gas price is, the faster it should get mined by the network.
  }
};
