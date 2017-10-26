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
    reverse: `翻轉訂單`, // Reverse Order
    stopped: `Parity的互動目前停止了重新整理和顯示Logs，請啟動它來檢視最新的更新。`,
    // Refresh and display of logs from Parity is currently stopped via the UI, start it to see the latest updates.
    title: `節點Logs` // Node Logs
  },
  miningSettings: {
    input: {
      author: {
        hint: `礦工名字`, // the mining author
        label: `礦工` // author
      },
      extradata: {
        hint: `提取挖到區塊的資料`, // extra data for mined blocks
        label: `提取資料` // extradata
      },
      gasFloor: {
        hint: `挖礦的gas下限目標`, // the gas floor target for mining
        label: `gas下限目標` // gas floor target
      },
      gasPrice: {
        hint: `挖礦的最低gas價格`, // the minimum gas price for mining
        label: `最低gas價格` // minimal gas price
      }
    },
    title: `挖礦設定` // mining settings
  },
  status: {
    hashrate: `{hashrate} H/s`, // {hashrate} H/s
    input: {
      chain: `鏈`, // chain
      enode: `enode`, // enode
      no: `否`, // no
      peers: `同步節點`, // peers
      port: `網路埠`, // network port
      rpcEnabled: `rpc開啟`, // rpc enabled
      rpcInterface: `rpc互動`, // rpc interface
      rpcPort: `rpc埠`, // rpc port
      yes: `是` // yes
    },
    title: {
      bestBlock: `最新區塊`, // best block
      hashRate: `雜湊率`, // hash rate
      network: `網路設定`, // network settings
      node: `節點`, // Node
      peers: `同步節點` // peers
    }
  },
  title: `狀態` // Status
};
