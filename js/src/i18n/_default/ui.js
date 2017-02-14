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
  verification: {
    gatherData: {
      termsOfService: `I agree to the terms and conditions below.`,
      isServerRunning: {
        true: `The verification server is running.`,
        false: `The verification server is not running.`,
        pending: `Checking if the verification server is running…`
      },
      nofee: `There is no additional fee.`,
      fee: `The additional fee is {amount} ETH.`,
      accountIsVerified: {
        true: `Your account is already verified.`,
        false: `Your account is not verified yet.`,
        pending: `Checking if your account is verified…`
      },
      accountHasRequested: {
        true: `You already requested verification from this account.`,
        false: `You did not request verification from this account yet.`,
        pending: `Checking if you requested verification…`
      },
      isAbleToRequest: {
        pending: `Validating your input…`
      },
      phoneNumber: {
        label: `phone number in international format`,
        hint: `the SMS will be sent to this number`
      },
      email: {
        label: `e-mail address`,
        hint: `the code will be sent to this address`
      }
    }
  },
  balance: {
    none: `There are no balances associated with this account`
  },
  blockStatus: {
    bestBlock: `{blockNumber} best block`,
    warpRestore: `{percentage}% warp restore`,
    syncStatus: `{currentBlock}/{highestBlock} syncing`,
    warpStatus: `, {percentage}% historic`
  },
  confirmDialog: {
    no: `no`,
    yes: `yes`
  },
  passwordStrength: {
    label: `password strength`
  },
  identityName: {
    unnamed: `UNNAMED`,
    null: `NULL`
  },
  txHash: {
    posted: `The transaction has been posted to the network with a hash of {hashLink}`,
    oog: `The transaction might have gone out of gas. Try again with more gas.`,
    waiting: `waiting for confirmations`,
    confirmations: `{count} {value, plural, one {confirmation} other {confirmations}}`
  }
};
