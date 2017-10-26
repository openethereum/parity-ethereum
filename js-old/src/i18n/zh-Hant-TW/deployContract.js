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
  busy: {
    title: `部署正在進行中`// The deployment is currently in progress
  },
  button: {
    cancel: `取消`, // Cancel
    close: `關閉`, // Close
    create: `建立`, // Create
    done: `完成`, // Done
    next: `下一個`// Next
  },
  completed: {
    description: `你的合約已經被部署在`// Your contract has been deployed at
  },
  details: {
    abi: {
      hint: `合約的abi或者solc 組合輸出`, // the abi of the contract to deploy or solc combined-output
      label: `abi / solc 組合輸出 `// abi / solc combined-output
    },
    address: {
      hint: `這個合約所有者的帳戶`, // the owner account for this contract
      label: `來自帳戶（合約所有者）`// from account (contract owner)
    },
    advanced: {
      label: `高階的傳送選項`// advanced sending options
    },
    amount: {
      hint: `轉到這個合約中的數額`, // the amount to transfer to the contract
      label: `傳送數額{tag}`// amount to transfer (in {tag})
    },
    code: {
      hint: `編譯好的合約程式碼`, // the compiled code of the contract to deploy
      label: `程式碼`// code
    },
    contract: {
      label: `選擇一個合約`// select a contract
    },
    description: {
      hint: `對這個合約的描述`, // a description for the contract
      label: `合約描述（可選）`// contract description (optional)
    },
    name: {
      hint: `已經部署合約的名字`, // a name for the deployed contract
      label: `合約名字`// contract name
    }
  },
  owner: {
    noneSelected: `選擇一個有效的地址作為合約的所有者`// a valid account as the contract owner needs to be selected
  },
  parameters: {
    choose: `選擇合約引數`// Choose the contract parameters
  },
  rejected: {
    description: `你可以安全地關閉視窗，合約部署不會發生。`, // You can safely close this window, the contract deployment will not occur.
    title: `部署已經被拒絕`// The deployment has been rejected
  },
  state: {
    completed: `合約部署已經完成`, // The contract deployment has been completed
    confirmationNeeded: `這一操作需要這個合約其他所有人的確認。`, // The operation needs confirmations from the other owners of the contract
    preparing: `為網路傳輸準備交易`, // Preparing transaction for network transmission
    validatingCode: `驗證已經部署的合約的程式碼`, // Validating the deployed contract code
    waitReceipt: `等待合約部署交易收據`, // Waiting for the contract deployment transaction receipt
    waitSigner: `等待Parity Secure Signer中的交易被確認 `// Waiting for confirmation of the transaction in the Parity Secure Signer
  },
  title: {
    completed: `完成`, // completed
    deployment: `部署`, // deployment
    details: `合約細節`, // contract details
    extras: `額外資訊`, // extra information
    failed: `部署失敗`, // deployment failed
    parameters: `s合約引數`, // contract parameter
    rejected: `拒絕`// rejected
  }
};
