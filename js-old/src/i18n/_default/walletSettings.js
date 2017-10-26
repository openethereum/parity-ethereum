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
  addOwner: {
    title: `Add Owner`
  },
  buttons: {
    cancel: `Cancel`,
    close: `Close`,
    next: `Next`,
    send: `Send`,
    sending: `Sending...`
  },
  changes: {
    modificationString: `For your modifications to be taken into account,
              other owners have to send the same modifications. They can paste
              this string to make it easier:`,
    none: `No modifications have been made to the Wallet settings.`,
    overview: `You are about to make the following modifications`
  },
  edit: {
    message: `In order to edit this contract's settings, at
                  least {owners, number} {owners, plural, one {owner } other {owners }} have to
                  send the very same modifications. You can paste a stringified version
                  of the modifications here.`
  },
  modifications: {
    daylimit: {
      hint: `amount of ETH spendable without confirmations`,
      label: `wallet day limit`
    },
    fromString: {
      label: `modifications`
    },
    owners: {
      label: `other wallet owners`
    },
    required: {
      hint: `number of required owners to accept a transaction`,
      label: `required owners`
    },
    sender: {
      hint: `send modifications as this owner`,
      label: `from account (wallet owner)`
    }
  },
  ownersChange: {
    details: `from {from} to {to}`,
    title: `Change Required Owners`
  },
  rejected: `The transaction #{txid} has been rejected`,
  removeOwner: {
    title: `Remove Owner`
  }
};
