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
    add: `新增`, // Add
    cancel: `取消`, // Cancel
    close: `關閉`, // Close
    create: `建立`, // Create
    done: `完成`, // Done
    next: `下一步`, // Next
    sending: `傳送中...` // Sending...
  },
  deployment: {
    message: `部署正在進行中` // The deployment is currently in progress
  },
  details: {
    address: {
      hint: `錢包的合約地址`, // the wallet contract address
      label: `錢包地址` // wallet address
    },
    dayLimitMulti: {
      hint: `無需確認即可使用的ETH數量`, // amount of ETH spendable without confirmations
      label: `錢包每日限額` // wallet day limit
    },
    description: {
      hint: `本地錢包描述`, // the local description for this wallet
      label: `錢包描述（可選）` // wallet description (optional)
    },
    descriptionMulti: {
      hint: `本地錢包描述`, // the local description for this wallet
      label: `錢包描述（可選）` // wallet description (optional)
    },
    name: {
      hint: `錢包本地名稱`,  // the local name for this wallet
      label: `錢包名稱` // wallet name
    },
    nameMulti: {
      hint: `錢包本地名稱`, // the local name for this wallet
      label: `錢包名稱` // wallet name
    },
    ownerMulti: {
      hint: `合約的持有者帳戶`, // the owner account for this contract
      label: `從帳戶 (contract owner)` // from account (contract owner)
    },
    ownersMulti: {
      label: `其他錢包持有者` // other wallet owners
    },
    ownersMultiReq: {
      hint: `接受交易所需的持有者人數`, // number of required owners to accept a transaction
      label: `所需持有者` // required owners
    }
  },
  info: {
    added: `已新增`, // added
    copyAddress: `複製地址至貼上板`, // copy address to clipboard
    created: `{name}已被{deployedOrAdded}至`, // {name} has been {deployedOrAdded} at
    dayLimit: `每日限額已被設定為{dayLimit}ETH`, // The daily limit is set to {dayLimit} ETH.
    deployed: `已部署`, // deployed
    numOwners: `需要{numOwners}個持有者才能確認一個交易`, // {numOwners} owners are required to confirm a transaction.
    owners: `以下為錢包持有者` // The following are wallet owners
  },
  rejected: {
    message: `部署被拒絕`, // The deployment has been rejected
    state: `錢包不會被建立。你可以安全地關閉本視窗`, // The wallet will not be created. You can safely close this window.
    title: `失敗` // rejected
  },
  states: {
    completed: `合約部署已完成`, // The contract deployment has been completed
    confirmationNeeded: `合約部署需要來自本錢包的其他持有者的確認`, // The contract deployment needs confirmations from other owners of the Wallet
    preparing: `交易正在準備被網路廣播`, // Preparing transaction for network transmission
    validatingCode: `正在驗證已部署的程式碼`, // Validating the deployed contract code
    waitingConfirm: `正在等待Parity Secure Signer確認本交易`, // Waiting for confirmation of the transaction in the Parity Secure Signer
    waitingReceipt: `正在等待合約部署交易收據` // Waiting for the contract deployment transaction receipt
  },
  steps: {
    deployment: `錢包部署`, // wallet deployment
    details: `錢包詳情`, // wallet details
    info: `錢包資訊`, // wallet informaton
    type: `錢包類別` // wallet type
  },
  type: {
    multisig: {
      description: `建立/部署一個{link}錢包`, // Create/Deploy a {link} Wallet
      label: `多重簽名錢包`, // Multi-Sig Wallet
      link: `標準多重簽名` // standard multi-signature
    },
    watch: {
      description: `新增一個已有錢包到你的帳戶`, // Add an existing wallet to your accounts
      label: `觀察錢包` // Watch a wallet
    }
  }
};
