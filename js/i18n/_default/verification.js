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
    cancel: `Cancel`,
    done: `Done`,
    next: `Next`
  },
  code: {
    error: `invalid code`,
    hint: `Enter the code you received.`,
    label: `verification code`,
    sent: `The verification code has been sent to {receiver}.`
  },
  confirmation: {
    authorise: `The verification code will be sent to the contract. Please authorize this using the Parity Signer.`,
    windowOpen: `Please keep this window open.`
  },
  done: {
    message: `Congratulations, your account is verified!`
  },
  email: {
    enterCode: `Enter the code you received via e-mail.`
  },
  gatherData: {
    email: {
      hint: `the code will be sent to this address`,
      label: `e-mail address`
    },
    phoneNumber: {
      hint: `the SMS will be sent to this number`,
      label: `phone number in international format`
    }
  },
  gatherDate: {
    email: {
      error: `invalid e-mail`
    },
    phoneNumber: {
      error: `invalid number`
    }
  },
  loading: `Loading verification data.`,
  request: {
    authorise: `A verification request will be sent to the contract. Please authorize this using the Parity Signer.`,
    requesting: `Requesting a code from the Parity server and waiting for the puzzle to be put into the contract.`,
    windowOpen: `Please keep this window open.`
  },
  sms: {
    enterCode: `Enter the code you received via SMS.`
  },
  steps: {
    code: `Enter Code`,
    completed: `Completed`,
    confirm: `Confirm`,
    data: `Enter Data`,
    method: `Method`,
    request: `Request`
  },
  title: `verify your account`,
  types: {
    email: {
      description: `The hash of the e-mail address you prove control over will be stored on the blockchain.`,
      label: `E-mail Verification`
    },
    sms: {
      description: `It will be stored on the blockchain that you control a phone number (not <em>which</em>).`,
      label: `SMS Verification`
    }
  }
};
