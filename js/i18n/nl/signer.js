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
  embedded: {
    noPending: `Er zijn momenteel geen lopende verzoeken die op je goedkeuring wachten`
  },
  mainDetails: {
    editTx: `Bewerk condities/gas/gasprijs`,
    tooltips: {
      total1: `De waarde van de transactie inclusief de miningskosten is {total} {type}.`,
      total2: `(Dit is inclusief een miners vergoeding van {fee} {token})`,
      value1: `De waarde van de transactie.`
    }
  },
  requestOrigin: {
    dapp: `door een dapp op {url}`,
    ipc: `via IPC sessie`,
    rpc: `via RPC {rpc}`,
    signerCurrent: `via huidige tab`,
    signerUI: `via UI sessie`,
    unknownInterface: `via onbekende interface`,
    unknownRpc: `niet geïdentificeerd`,
    unknownUrl: `onbekende URL`
  },
  requestsPage: {
    noPending: `Er zijn geen verzoeken die je goedkeuring vereisen.`,
    pendingTitle: `Openstaande Verzoeken`,
    queueTitle: `Lokale Transacties`
  },
  sending: {
    hardware: {
      confirm: `Bevestig de transactie op je aangesloten hardware wallet`,
      connect: `Sluit je hardware wallet aan voordat je de transactie bevestigd`
    }
  },
  signRequest: {
    request: `Een verzoek om data te ondertekenen met jouw account:`,
    state: {
      confirmed: `Bevestigd`,
      rejected: `Afgewezen`
    },
    unknownBinary: `(Onbekende binary data)`,
    warning: `WAARSCHUWING: Deze gevolgen hiervan kunnen ernstig zijn. Bevestig het verzoek alleen als je het zeker weet.`
  },
  title: `Trusted Signer`,
  txPending: {
    buttons: {
      viewToggle: `bekijk transactie`
    }
  },
  txPendingConfirm: {
    buttons: {
      confirmBusy: `Bevestigen...`,
      confirmRequest: `Bevestig Verzoek`
    },
    errors: {
      invalidWallet: `Opgegeven wallet bestand is ongeldig.`
    },
    password: {
      decrypt: {
        hint: `open (decrypt) de sleutel met je wachtwoord`,
        label: `Sleutel Wachtwoord`
      },
      unlock: {
        hint: `ontgrendel het account`,
        label: `Account Wachtwoord`
      }
    },
    passwordHint: `(hint) {passwordHint}`,
    selectKey: {
      hint: `De sleutelbestand (keyfile) die je voor dit account wilt gebruiken`,
      label: `Selecteer Lokale Sleutel (key)`
    },
    tooltips: {
      password: `Geef een wachtwoord voor dit account`
    }
  },
  txPendingForm: {
    changedMind: `Ik heb me bedacht`,
    reject: `wijs verzoek af`
  },
  txPendingReject: {
    buttons: {
      reject: `Wijs Verzoek Af`
    },
    info: `Weet je zeker dat je dit verzoek wilt afwijzen?`,
    undone: `Dit kan niet ongedaan gemaakt worden`
  }
};
