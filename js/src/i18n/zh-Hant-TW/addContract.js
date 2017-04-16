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
  abi: {
    hint: `合約的ABI`, // the abi for the contract
    label: `合約ABI` // contract abi
  },
  abiType: {
    custom: {
      description: `通過自定義ABI創造的合約`, // Contract created from custom ABI
      label: `自定義合約` // Custom Contract
    },
    multisigWallet: {
      description: `以太坊多重簽名合約{link}`, // Ethereum Multisig contract {link}
      label: `多重簽名錢包`, // Multisig Wallet
      link: `參考合約程式碼` // see contract code
    },
    token: {
      description: `一個標準的{erc20}代幣`, // A standard {erc20} token
      erc20: `ERC 20`, // ERC 20
      label: `代幣` // Token
    }
  },
  address: {
    hint: `合約的網路地址`, // the network address for the contract
    label: `網路地址` // network address
  },
  button: {
    add: `新增合約`, // Add Contract
    cancel: `取消`, // Cancel
    next: `下一步`, // Next
    prev: `上一步` // Back
  },
  description: {
    hint: `記錄的詳細描述`, // an expanded description for the entry
    label: `（可選）合約描述` // (optional) contract description
  },
  name: {
    hint: `合約的描述性名稱`, // a descriptive name for the contract
    label: `合約名` // contract name
  },
  title: {
    details: `輸入合約細節`, // enter contract details
    type: `選擇合約種類` // choose a contract type
  }
};
