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
    autoCompile: `自動編譯`, // Auto-Compile
    compile: `編譯`, // Compile
    deploy: `部署`, // Deploy
    import: `載入Solidity`, // Import Solidity
    load: `載入`, // Load
    new: `新建`, // New
    optimise: `優化`, // Optimise
    save: `儲存` // Save
  },
  compiling: {
    action: `請編譯原始碼`, // Please compile the source code.
    busy: `編譯中...` // Compiling...
  },
  details: {
    saved: `(已儲存 {timestamp})` // (saved {timestamp})
  },
  error: {
    noContract: `沒有找到合約`, // No contract has been found.
    params: `發生瞭如下描述的一個錯誤` // An error occurred with the following description
  },
  input: {
    abi: `ABI介面`, // ABI Interface
    code: `位元組碼`, // Bytecode
    metadata: `元資料`, // Metadata
    swarm: `Swarm元資料雜湊` // Sarm Metadata Hash
  },
  title: {
    contract: `選擇一個合約`, // Select a contract
    loading: `載入中...`, // Loading...
    main: `寫一個合約`, // Write a Contract
    messages: `編譯器訊息`, // Compiler messages
    new: `新建Solidity合約`, // New Solidity Contract
    parameters: `變數`, // Parameters
    saved: `已儲存 @ {timestamp}`, // saved @ {timestamp}
    selectSolidity: `選擇Solidity版本`, // Select a Solidity version
    solidity: `正在載入Solidity {version}` // Loading Solidity {version}
  },
  type: {
    humanErc20: `Human代幣合約編碼`, // Implementation of the Human Token Contract
    implementErc20: `ERC20代幣合約編碼`, // Implementation of the ERC20 Token Contract
    multisig: `多籤錢包編碼`, // Implementation of a multisig Wallet
    standardErc20: `標準ERC20代幣合約` // Standard ERC20 Token Contract
  }
};
