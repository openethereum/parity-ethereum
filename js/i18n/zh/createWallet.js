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
  button: {
    add: `添加`, // Add
    cancel: `取消`, // Cancel
    close: `关闭`, // Close
    create: `创建`, // Create
    done: `完成`, // Done
    next: `下一步`, // Next
    sending: `发送中...` // Sending...
  },
  deployment: {
    message: `部署正在进行中` // The deployment is currently in progress
  },
  details: {
    address: {
      hint: `钱包的合约地址`, // the wallet contract address
      label: `钱包地址` // wallet address
    },
    dayLimitMulti: {
      hint: `无需确认即可使用的ETH数量`, // amount of ETH spendable without confirmations
      label: `钱包每日限额` // wallet day limit
    },
    description: {
      hint: `本地钱包描述`, // the local description for this wallet
      label: `钱包描述（可选）` // wallet description (optional)
    },
    descriptionMulti: {
      hint: `本地钱包描述`, // the local description for this wallet
      label: `钱包描述（可选）` // wallet description (optional)
    },
    name: {
      hint: `钱包本地名称`,  // the local name for this wallet
      label: `钱包名称` // wallet name
    },
    nameMulti: {
      hint: `钱包本地名称`, // the local name for this wallet
      label: `钱包名称` // wallet name
    },
    ownerMulti: {
      hint: `合约的持有者账户`, // the owner account for this contract
      label: `从账户 (contract owner)` // from account (contract owner)
    },
    ownersMulti: {
      label: `其他钱包持有者` // other wallet owners
    },
    ownersMultiReq: {
      hint: `接受交易所需的持有者人数`, // number of required owners to accept a transaction
      label: `所需持有者` // required owners
    }
  },
  info: {
    added: `已添加`, // added
    copyAddress: `复制地址至粘贴板`, // copy address to clipboard
    created: `{name}已被{deployedOrAdded}至`, // {name} has been {deployedOrAdded} at
    dayLimit: `每日限额已被设置为{dayLimit}ETH`, // The daily limit is set to {dayLimit} ETH.
    deployed: `已部署`, // deployed
    numOwners: `需要{numOwners}个持有者才能确认一个交易`, // {numOwners} owners are required to confirm a transaction.
    owners: `以下为钱包持有者` // The following are wallet owners
  },
  rejected: {
    message: `部署被拒绝`, // The deployment has been rejected
    state: `钱包不会被创建。你可以安全地关闭本窗口`, // The wallet will not be created. You can safely close this window.
    title: `失败` // rejected
  },
  states: {
    completed: `合约部署已完成`, // The contract deployment has been completed
    confirmationNeeded: `合约部署需要来自本钱包的其他持有者的确认`, // The contract deployment needs confirmations from other owners of the Wallet
    preparing: `交易正在准备被网络广播`, // Preparing transaction for network transmission
    validatingCode: `正在验证已部署的代码`, // Validating the deployed contract code
    waitingConfirm: `正在等待Parity Secure Signer确认本交易`, // Waiting for confirmation of the transaction in the Parity Secure Signer
    waitingReceipt: `正在等待合约部署交易收据` // Waiting for the contract deployment transaction receipt
  },
  steps: {
    deployment: `钱包部署`, // wallet deployment
    details: `钱包详情`, // wallet details
    info: `钱包信息`, // wallet informaton
    type: `钱包类别` // wallet type
  },
  type: {
    multisig: {
      description: `创建/部署一个{link}钱包`, // Create/Deploy a {link} Wallet
      label: `多重签名钱包`, // Multi-Sig Wallet
      link: `标准多重签名` // standard multi-signature
    },
    watch: {
      description: `添加一个已有钱包到你的账户`, // Add an existing wallet to your accounts
      label: `观察钱包` // Watch a wallet
    }
  }
};
