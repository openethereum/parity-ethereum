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
    error: `无效验证码`, // invalid code
    hint: `输入你收到的验证码。`, // Enter the code you received.
    label: `验证码`, // verification code
    sent: `验证码被发送到接收者{receiver}.`// The verification code has been sent to {receiver}
  },
  confirmation: {
    authorise: `验证码将被发送到合约。请使用Parity Signer进行授权。`, // The verification code will be sent to the contract. Please authorize this using the Parity Signer.
    windowOpen: `请保持这个窗口打开状态。`// Please keep this window open.
  },
  done: {
    message: `恭喜，你的账户已经被认证。`// Congratulations, your account is verified!
  },
  email: {
    enterCode: `输入你从邮箱获得验证码。`// Enter the code you received via e-mail.
  },
  gatherData: {
    email: {
      hint: `验证码将被发送到这个地址`, // the code will be sent to this address
      label: `邮箱地址`// e-mail address
    },
    phoneNumber: {
      hint: `短信将被发送到这个号码`, // the SMS will be sent to this number
      label: `国际格式的手机号码`// phone number in international format
    }
  },
  gatherDate: {
    email: {
      error: `无效邮箱`// invalid e-mail
    },
    phoneNumber: {
      error: `无效数字`// invalid number
    }
  },
  loading: `加载验证数据`, // Loading verification data.
  request: {
    authorise: `验证请求将被发送到这个合约。请使用Parity Signer进行授权。`, // A verification request will be sent to the contract. Please authorize this using the Parity Signer.
    requesting: `正在从Parity服务器请求一个验证码，等待它被输入到合约。`, // Requesting a code from the Parity server and waiting for the puzzle to be put into the contract.
    windowOpen: `请保持窗口为打开状态。`// Please keep this window open.
  },
  sms: {
    enterCode: `输入你从短信收到的验证码。`// Enter the code you received via SMS.
  },
  steps: {
    code: `输入验证码`, // Enter Code
    completed: `完成`, // Completed
    confirm: `确认`, // Confirm
    data: `输入数据`, // Enter Data
    method: `方式`, // Method
    request: `请求`// Request
  },
  title: `t验证你的账户`, // verify your accoun
  types: {
    email: {
      description: `你所控制的邮箱地址的哈希值将被存储在区块链。`, // The hash of the e-mail address you prove control over will be stored on the blockchain.
      label: `邮箱验证`// E-mail Verification
    },
    sms: {
      description: `你所控制的手机号码将被存储在区块链。`, // It will be stored on the blockchain that you control a phone number (not <em>which</em>).
      label: `短信验证`// SMS Verification
    }
  }
};
