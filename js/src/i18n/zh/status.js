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
    reverse: `翻转订单`, // Reverse Order
    stopped: `Parity的交互目前停止了刷新和显示Logs，请启动它来查看最新的更新。`,
    // Refresh and display of logs from Parity is currently stopped via the UI, start it to see the latest updates.
    title: `节点Logs` // Node Logs
  },
  miningSettings: {
    input: {
      author: {
        hint: `矿工名字`, // the mining author
        label: `矿工` // author
      },
      extradata: {
        hint: `提取挖到区块的数据`, // extra data for mined blocks
        label: `提取数据` // extradata
      },
      gasFloor: {
        hint: `挖矿的gas下限目标`, // the gas floor target for mining
        label: `gas下限目标` // gas floor target
      },
      gasPrice: {
        hint: `挖矿的最低gas价格`, // the minimum gas price for mining
        label: `最低gas价格` // minimal gas price
      }
    },
    title: `挖矿设置` // mining settings
  },
  status: {
    hashrate: `{hashrate} H/s`, // {hashrate} H/s
    input: {
      chain: `链`, // chain
      enode: `enode`, // enode
      no: `否`, // no
      peers: `同步节点`, // peers
      port: `网络端口`, // network port
      rpcEnabled: `rpc开启`, // rpc enabled
      rpcInterface: `rpc交互`, // rpc interface
      rpcPort: `rpc端口`, // rpc port
      yes: `是` // yes
    },
    title: {
      bestBlock: `最新区块`, // best block
      hashRate: `哈希率`, // hash rate
      network: `网络设置`, // network settings
      node: `节点`, // Node
      peers: `同步节点` // peers
    }
  },
  title: `状态` // Status
};
