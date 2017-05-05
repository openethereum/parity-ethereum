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
  buttons: {
    autoCompile: `自动编译`, // Auto-Compile
    compile: `编译`, // Compile
    deploy: `部署`, // Deploy
    import: `载入Solidity`, // Import Solidity
    load: `加载`, // Load
    new: `新建`, // New
    optimise: `优化`, // Optimise
    save: `保存` // Save
  },
  compiling: {
    action: `请编译源代码`, // Please compile the source code.
    busy: `编译中...` // Compiling...
  },
  details: {
    saved: `(已保存 {timestamp})` // (saved {timestamp})
  },
  error: {
    noContract: `没有找到合约`, // No contract has been found.
    params: `发生了如下描述的一个错误` // An error occurred with the following description
  },
  input: {
    abi: `ABI界面`, // ABI Interface
    code: `字节码`, // Bytecode
    metadata: `元数据`, // Metadata
    swarm: `Swarm元数据哈希` // Sarm Metadata Hash
  },
  title: {
    contract: `选择一个合约`, // Select a contract
    loading: `加载中...`, // Loading...
    main: `写一个合约`, // Write a Contract
    messages: `编译器消息`, // Compiler messages
    new: `新建Solidity合约`, // New Solidity Contract
    parameters: `变量`, // Parameters
    saved: `已保存 @ {timestamp}`, // saved @ {timestamp}
    selectSolidity: `选择Solidity版本`, // Select a Solidity version
    solidity: `正在加载Solidity {version}` // Loading Solidity {version}
  },
  type: {
    humanErc20: `Human代币合约编码`, // Implementation of the Human Token Contract
    implementErc20: `ERC20代币合约编码`, // Implementation of the ERC20 Token Contract
    multisig: `多签钱包编码`, // Implementation of a multisig Wallet
    standardErc20: `标准ERC20代币合约` // Standard ERC20 Token Contract
  }
};
