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
    awaitingConfirmation: `Awaiting confirmation of the deposit address for your {typeSymbol} funds exchange`,
    awaitingDeposit: `{shapeshiftLink} is awaiting a {typeSymbol} deposit. Send the funds from your {typeSymbol} network client to -`,
    minimumMaximum: `{minimum} minimum, {maximum} maximum`
  },
  awaitingExchangeStep: {
    receivedInfo: `{shapeshiftLink} has received a deposit of -`,
    awaitingCompletion: `Awaiting the completion of the funds exchange and transfer of funds to your Parity account.`
  },
  completedStep: {
    completed: `{shapeshiftLink} has completed the funds exchange.`,
    parityFunds: `The change in funds will be reflected in your Parity account shortly.`
  },
  errorStep: {
    info: `The funds shifting via {shapeshiftLink} failed with a fatal error on the exchange. The error message received from the exchange is as follow:`
  },
  warning: {
    noPrice: `No price match was found for the selected type`
  },
  optionsStep: {
    noPairs: `There are currently no exchange pairs/coins available to fund with.`,
    typeSelect: {
      hint: `the type of crypto conversion to do`,
      label: `fund account from`
    },
    returnAddr: {
      hint: `the return address for send failures`,
      label: `(optional) {coinSymbol} return address`
    },
    terms: {
      label: `I understand that ShapeShift.io is a 3rd-party service and by using the service any transfer of information and/or funds is completely out of the control of Parity`
    }
  },
  price: {
    minMax: `({minimum} minimum, {maximum} maximum)`
  },
  title: {
    details: `details`,
    deposit: `awaiting deposit`,
    exchange: `awaiting exchange`,
    completed: `completed`,
    error: `exchange failed`
  },
  button: {
    cancel: `Cancel`,
    shift: `Shift Funds`,
    done: `Close`
  }
};
