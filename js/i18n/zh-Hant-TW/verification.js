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
    cancel: `取消`, // Cancel
    done: `完成`, // Done
    next: `下一步`// Next
  },
  code: {
    error: `無效驗證碼`, // invalid code
    hint: `輸入你收到的驗證碼。`, // Enter the code you received.
    label: `驗證碼`, // verification code
    sent: `驗證碼被髮送到接收者{receiver}.`// The verification code has been sent to {receiver}
  },
  confirmation: {
    authorise: `驗證碼將被髮送到合約。請使用Parity Signer進行授權。`, // The verification code will be sent to the contract. Please authorize this using the Parity Signer.
    windowOpen: `請保持這個視窗開啟狀態。`// Please keep this window open.
  },
  done: {
    message: `恭喜，你的帳戶已經被認證。`// Congratulations, your account is verified!
  },
  email: {
    enterCode: `輸入你從郵箱獲得驗證碼。`// Enter the code you received via e-mail.
  },
  gatherData: {
    email: {
      hint: `驗證碼將被髮送到這個地址`, // the code will be sent to this address
      label: `郵箱地址`// e-mail address
    },
    phoneNumber: {
      hint: `簡訊將被髮送到這個號碼`, // the SMS will be sent to this number
      label: `國際格式的手機號碼`// phone number in international format
    }
  },
  gatherDate: {
    email: {
      error: `無效郵箱`// invalid e-mail
    },
    phoneNumber: {
      error: `無效數字`// invalid number
    }
  },
  loading: `載入驗證資料`, // Loading verification data.
  request: {
    authorise: `驗證請求將被髮送到這個合約。請使用Parity Signer進行授權。`, // A verification request will be sent to the contract. Please authorize this using the Parity Signer.
    requesting: `正在從Parity伺服器請求一個驗證碼，等待它被輸入到合約。`, // Requesting a code from the Parity server and waiting for the puzzle to be put into the contract.
    windowOpen: `請保持視窗為開啟狀態。`// Please keep this window open.
  },
  sms: {
    enterCode: `輸入你從簡訊收到的驗證碼。`// Enter the code you received via SMS.
  },
  steps: {
    code: `輸入驗證碼`, // Enter Code
    completed: `完成`, // Completed
    confirm: `確認`, // Confirm
    data: `輸入資料`, // Enter Data
    method: `方式`, // Method
    request: `請求`// Request
  },
  title: `t驗證你的帳戶`, // verify your accoun
  types: {
    email: {
      description: `你所控制的郵箱地址的雜湊值將被儲存在區塊鏈。`, // The hash of the e-mail address you prove control over will be stored on the blockchain.
      label: `郵箱驗證`// E-mail Verification
    },
    sms: {
      description: `你所控制的手機號碼將被儲存在區塊鏈。`, // It will be stored on the blockchain that you control a phone number (not <em>which</em>).
      label: `簡訊驗證`// SMS Verification
    }
  }
};
