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
    hint: `合约的ABI`, // the abi for the contract
    label: `合约ABI` // contract abi
  },
  abiType: {
    custom: {
      description: `通过自定义ABI创造的合约`, // Contract created from custom ABI
      label: `自定义合约` // Custom Contract
    },
    multisigWallet: {
      description: `以太坊多重签名合约{link}`, // Ethereum Multisig contract {link}
      label: `多重签名钱包`, // Multisig Wallet
      link: `参考合约代码` // see contract code
    },
    token: {
      description: `一个标准的{erc20}代币`, // A standard {erc20} token
      erc20: `ERC 20`, // ERC 20
      label: `代币` // Token
    }
  },
  address: {
    hint: `合约的网络地址`, // the network address for the contract
    label: `网络地址` // network address
  },
  button: {
    add: `添加合约`, // Add Contract
    cancel: `取消`, // Cancel
    next: `下一步`, // Next
    prev: `上一步` // Back
  },
  description: {
    hint: `记录的详细描述`, // an expanded description for the entry
    label: `（可选）合约描述` // (optional) contract description
  },
  name: {
    hint: `合约的描述性名称`, // a descriptive name for the contract
    label: `合约名` // contract name
  },
  title: {
    details: `输入合约细节`, // enter contract details
    type: `选择合约种类` // choose a contract type
  }
};
