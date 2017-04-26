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
    title: `部署正在进行中`// The deployment is currently in progress
  },
  button: {
    cancel: `取消`, // Cancel
    close: `关闭`, // Close
    create: `创建`, // Create
    done: `完成`, // Done
    next: `下一个`// Next
  },
  completed: {
    description: `你的合约已经被部署在`// Your contract has been deployed at
  },
  details: {
    abi: {
      hint: `合约的abi或者solc 组合输出`, // the abi of the contract to deploy or solc combined-output
      label: `abi / solc 组合输出 `// abi / solc combined-output
    },
    address: {
      hint: `这个合约所有者的账户`, // the owner account for this contract
      label: `来自账户（合约所有者）`// from account (contract owner)
    },
    advanced: {
      label: `高级的发送选项`// advanced sending options
    },
    amount: {
      hint: `转到这个合约中的数额`, // the amount to transfer to the contract
      label: `发送数额{tag}`// amount to transfer (in {tag})
    },
    code: {
      hint: `编译好的合约代码`, // the compiled code of the contract to deploy
      label: `代码`// code
    },
    contract: {
      label: `选择一个合约`// select a contract
    },
    description: {
      hint: `对这个合约的描述`, // a description for the contract
      label: `合约描述（可选）`// contract description (optional)
    },
    name: {
      hint: `已经部署合约的名字`, // a name for the deployed contract
      label: `合约名字`// contract name
    }
  },
  owner: {
    noneSelected: `选择一个有效的地址作为合约的所有者`// a valid account as the contract owner needs to be selected
  },
  parameters: {
    choose: `选择合约参数`// Choose the contract parameters
  },
  rejected: {
    description: `你可以安全地关闭窗口，合约部署不会发生。`, // You can safely close this window, the contract deployment will not occur.
    title: `部署已经被拒绝`// The deployment has been rejected
  },
  state: {
    completed: `合约部署已经完成`, // The contract deployment has been completed
    confirmationNeeded: `这一操作需要这个合约其他所有人的确认。`, // The operation needs confirmations from the other owners of the contract
    preparing: `为网络传输准备交易`, // Preparing transaction for network transmission
    validatingCode: `验证已经部署的合约的代码`, // Validating the deployed contract code
    waitReceipt: `等待合约部署交易收据`, // Waiting for the contract deployment transaction receipt
    waitSigner: `等待Parity Secure Signer中的交易被确认 `// Waiting for confirmation of the transaction in the Parity Secure Signer
  },
  title: {
    completed: `完成`, // completed
    deployment: `部署`, // deployment
    details: `合约细节`, // contract details
    extras: `额外信息`, // extra information
    failed: `部署失败`, // deployment failed
    parameters: `s合约参数`, // contract parameter
    rejected: `拒绝`// rejected
  }
};
