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
  decryptRequest: {
    request: `A request to decrypt data using your account:`,
    state: {
      confirmed: `Confirmed`,
      rejected: `Rejected`
    }
  },
  embedded: {
    noPending: `There are currently no pending requests awaiting your confirmation`
  },
  mainDetails: {
    editTx: `Edit conditions/gas/gasPrice`,
    tooltips: {
      total1: `The value of the transaction including the mining fee is {total} {type}.`,
      total2: `(This includes a mining fee of {fee} {token})`,
      value1: `The value of the transaction.`
    }
  },
  requestOrigin: {
    dapp: `by a dapp at {url}`,
    ipc: `via IPC session`,
    rpc: `via RPC {url}`,
    signerCurrent: `via current tab`,
    signerUI: `via UI session`,
    unknownInterface: `via unknown interface`,
    unknownRpc: `unidentified`,
    unknownUrl: `unknown URL`
  },
  requestsPage: {
    noPending: `There are no requests requiring your confirmation.`,
    pendingTitle: `Pending Signature Authorization`,
    queueTitle: `Local Transactions`
  },
  sending: {
    external: {
      scanSigned: `Scan the QR code of the signed transaction from your external device`,
      scanTx: `Please scan the transaction QR on your external device`
    },
    hardware: {
      confirm: `Please confirm the transaction on your attached hardware device`,
      connect: `Please attach your hardware device before confirming the transaction`
    }
  },
  signRequest: {
    request: `A request to sign data using your account:`,
    state: {
      confirmed: `Confirmed`,
      rejected: `Rejected`
    },
    tooltip: {
      data: `Data: {data}`,
      hash: `Hash to be signed: {hashToSign}`
    },
    unknownBinary: `(Unknown binary data)`,
    warning: `WARNING: The consequences of doing this may be grave. Confirm the request only if you are sure.`
  },
  title: `Trusted Signer`,
  txPending: {
    buttons: {
      viewToggle: `view transaction`
    }
  },
  txPendingConfirm: {
    buttons: {
      confirmBusy: `Confirming...`,
      confirmRequest: `Confirm Request`,
      scanSigned: `Scan Signed QR`
    },
    errors: {
      invalidWallet: `Given wallet file is invalid.`
    },
    password: {
      decrypt: {
        hint: `decrypt the key`,
        label: `Key Password`
      },
      unlock: {
        hint: `unlock the account`,
        label: `Account Password`
      }
    },
    passwordHint: `(hint) {passwordHint}`,
    selectKey: {
      hint: `The keyfile to use for this account`,
      label: `Select Local Key`
    },
    tooltips: {
      password: `Please provide a password for this account`
    }
  },
  txPendingForm: {
    changedMind: `I've changed my mind`,
    reject: `reject request`
  },
  txPendingReject: {
    buttons: {
      reject: `Reject Request`
    },
    info: `Are you sure you want to reject request?`,
    undone: `This cannot be undone`
  }
};
