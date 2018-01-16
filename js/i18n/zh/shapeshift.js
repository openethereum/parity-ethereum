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
    awaitingConfirmation: `正在等待你的{typeSymbol}资金交易的存款地址的确认信息`,
    // Awaiting confirmation of the deposit address for your {typeSymbol} funds exchange
    awaitingDeposit: `{shapeshiftLink}正在等待{typeSymbol}的存入.从你的{typeSymbol}网络客户端发送资金至-`,
    // {shapeshiftLink} is awaiting a {typeSymbol} deposit. Send the funds from your {typeSymbol} network client to -
    minimumMaximum: `{minimum}至少, {maximum}至多` // {minimum} minimum, {maximum} maximum
  },
  awaitingExchangeStep: {
    awaitingCompletion: `正在完成资金交易并发送资金至你的Parity账户`,
    // Awaiting the completion of the funds exchange and transfer of funds to your Parity account.
    receivedInfo: `{shapeshiftLink}已经收到存款-`
    // {shapeshiftLink} has received a deposit of -
  },
  button: {
    cancel: `取消`, // Cancel
    done: `关闭`, // Close
    shift: `转换资金` // Shift Funds
  },
  completedStep: {
    completed: `{shapeshiftLink}已经完成了资金交易。`, // {shapeshiftLink} has completed the funds exchange.
    parityFunds: `资金的改变会马上在你的Parity账户里体现。`
    // The change in funds will be reflected in your Parity account shortly.
  },
  errorStep: {
    info: `通过{shapeshiftLink}进行的资金转换因为一个交易的致命错误失败了。交易提供的错误信息如下：`
    // The funds shifting via {shapeshiftLink} failed with a fatal error on the exchange. The error message received from the exchange
    // is as follow:
  },
  optionsStep: {
    noPairs: `目前没有可匹配的交易/货币可用来进行转换`,
    // There are currently no exchange pairs/coins available to fund with.
    returnAddr: {
      hint: `转换错误后的发回地址`, // the return address for send failures
      label: `（可选）{coinSymbol}发回地址` // (optional) {coinSymbol} return address
    },
    terms: {
      label: `我理解ShapeShift.io是一个第三方服务，使用此服务发生的任何信息/资金发送是完全不受Parity控制的`
      // I understand that ShapeShift.io is a 3rd-party service and by using the service any transfer of information and/or funds is
      // completely out of the control of Parity
    },
    typeSelect: {
      hint: `数字货币转换的种类`, // the type of crypto conversion to do
      label: `来自资金账户` // fund account from
    }
  },
  price: {
    minMax: `({minimum}至小, {maximum}至大)` // ({minimum} minimum, {maximum} maximum)
  },
  title: {
    completed: `完成`, // completed
    deposit: `等待存款`, // awaiting deposit
    details: `详情`, // details
    error: `交易失败`, // exchange failed
    exchange: `等待交易` // awaiting exchange
  },
  warning: {
    noPrice: `所选择的类型没有匹配的价格` // No price match was found for the selected type
  }
};
