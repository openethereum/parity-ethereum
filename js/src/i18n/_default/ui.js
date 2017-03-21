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
  balance: {
    none: `There are no balances associated with this account`
  },
  blockStatus: {
    bestBlock: `{blockNumber} best block`,
    syncStatus: `{currentBlock}/{highestBlock} syncing`,
    warpRestore: `{percentage}% warp restore`,
    warpStatus: `, {percentage}% historic`
  },
  confirmDialog: {
    no: `no`,
    yes: `yes`
  },
  identityName: {
    null: `NULL`,
    unnamed: `UNNAMED`
  },
  passwordStrength: {
    label: `password strength`
  },
  tooltips: {
    button: {
      done: `Done`,
      next: `Next`,
      skip: `Skip`
    }
  },
  txHash: {
    confirmations: `{count} {value, plural, one {confirmation} other {confirmations}}`,
    oog: `The transaction might have gone out of gas. Try again with more gas.`,
    posted: `The transaction has been posted to the network with a hash of {hashLink}`,
    waiting: `waiting for confirmations`
  },
  verification: {
    gatherData: {
      accountHasRequested: {
        false: `You did not request verification from this account yet.`,
        pending: `Checking if you requested verification…`,
        true: `You already requested verification from this account.`
      },
      accountIsVerified: {
        false: `Your account is not verified yet.`,
        pending: `Checking if your account is verified…`,
        true: `Your account is already verified.`
      },
      email: {
        hint: `the code will be sent to this address`,
        label: `e-mail address`
      },
      fee: `The additional fee is {amount} ETH.`,
      isAbleToRequest: {
        pending: `Validating your input…`
      },
      isServerRunning: {
        false: `The verification server is not running.`,
        pending: `Checking if the verification server is running…`,
        true: `The verification server is running.`
      },
      nofee: `There is no additional fee.`,
      phoneNumber: {
        hint: `the SMS will be sent to this number`,
        label: `phone number in international format`
      },
      termsOfService: `I agree to the terms and conditions below.`
    }
  }
};
