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
  awaitingDepositStep: {
    awaitingConfirmation: `正在等待你的{typeSymbol}資金交易的存款地址的確認資訊`,
    // Awaiting confirmation of the deposit address for your {typeSymbol} funds exchange
    awaitingDeposit: `{shapeshiftLink}正在等待{typeSymbol}的存入.從你的{typeSymbol}網路客戶端傳送資金至-`,
    // {shapeshiftLink} is awaiting a {typeSymbol} deposit. Send the funds from your {typeSymbol} network client to -
    minimumMaximum: `{minimum}至少, {maximum}至多` // {minimum} minimum, {maximum} maximum
  },
  awaitingExchangeStep: {
    awaitingCompletion: `正在完成資金交易併發送資金至你的Parity帳戶`,
    // Awaiting the completion of the funds exchange and transfer of funds to your Parity account.
    receivedInfo: `{shapeshiftLink}已經收到存款-`
    // {shapeshiftLink} has received a deposit of -
  },
  button: {
    cancel: `取消`, // Cancel
    done: `關閉`, // Close
    shift: `轉換資金` // Shift Funds
  },
  completedStep: {
    completed: `{shapeshiftLink}已經完成了資金交易。`, // {shapeshiftLink} has completed the funds exchange.
    parityFunds: `資金的改變會馬上在你的Parity帳戶裡體現。`
    // The change in funds will be reflected in your Parity account shortly.
  },
  errorStep: {
    info: `通過{shapeshiftLink}進行的資金轉換因為一個交易的致命錯誤失敗了。交易提供的錯誤資訊如下：`
    // The funds shifting via {shapeshiftLink} failed with a fatal error on the exchange. The error message received from the exchange
    // is as follow:
  },
  optionsStep: {
    noPairs: `目前沒有可匹配的交易/貨幣可用來進行轉換`,
    // There are currently no exchange pairs/coins available to fund with.
    returnAddr: {
      hint: `轉換錯誤後的發回地址`, // the return address for send failures
      label: `（可選）{coinSymbol}發回地址` // (optional) {coinSymbol} return address
    },
    terms: {
      label: `我理解ShapeShift.io是一個第三方服務，使用此服務發生的任何資訊/資金髮送是完全不受Parity控制的`
      // I understand that ShapeShift.io is a 3rd-party service and by using the service any transfer of information and/or funds is
      // completely out of the control of Parity
    },
    typeSelect: {
      hint: `數字貨幣轉換的種類`, // the type of crypto conversion to do
      label: `來自資金帳戶` // fund account from
    }
  },
  price: {
    minMax: `({minimum}至小, {maximum}至大)` // ({minimum} minimum, {maximum} maximum)
  },
  title: {
    completed: `完成`, // completed
    deposit: `等待存款`, // awaiting deposit
    details: `詳情`, // details
    error: `交易失敗`, // exchange failed
    exchange: `等待交易` // awaiting exchange
  },
  warning: {
    noPrice: `所選擇的型別沒有匹配的價格` // No price match was found for the selected type
  }
};
