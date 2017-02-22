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
    none: `Er zijn geen tegoeden gekoppeld aan dit account`
  },
  blockStatus: {
    bestBlock: `{blockNumber} beste blok`,
    syncStatus: `{currentBlock}/{highestBlock} synchroniseren`,
    warpRestore: `{percentage}% warp restore`,
    warpStatus: `, {percentage}% historic`
  },
  confirmDialog: {
    no: `nee`,
    yes: `ja`
  },
  identityName: {
    null: `NUL`,
    unnamed: `NAAMLOOS`
  },
  passwordStrength: {
    label: `wachtwoord sterkte`
  },
  tooltips: {
    button: {
      done: `Klaar`,
      next: `Volgende`,
      skip: `Overslaan`
    }
  },
  txHash: {
    confirmations: `{count} {value, plural, one {confirmation} other {confirmations}}`,
    oog: `De transactie heeft misschien al zijn gas verbruikt. Probeer het opnieuw met meer gas.`,
    posted: `De transactie is op het netwerk geplaatst met hash {hashLink}`,
    waiting: `wachten op bevestigingen`
  },
  verification: {
    gatherData: {
      accountHasRequested: {
        false: `Je hebt nog geen verificatie aangevraagd voor dit account.`,
        pending: `Aan het controleren of je verificatie hebt aangevraagd…`,
        true: `Je hebt al verificatie aangevraagd voor dit account.`
      },
      accountIsVerified: {
        false: `Je account is nog niet geverifieerd`,
        pending: `Aan het controleren of je account is geverifieerd…`,
        true: `Je account is al geverifieerd.`
      },
      email: {
        hint: `de code zal naar dit adres worden verzonden`,
        label: `e-mail adres`
      },
      fee: `De extra vergoeding is {amount} ETH.`,
      isAbleToRequest: {
        pending: `Valideren van je invoer…`
      },
      isServerRunning: {
        false: `De verificatie server is niet actief.`,
        pending: `Controleren of de verificatie server actief is…`,
        true: `De verificatie server is actief.`
      },
      nofee: `Er zijn geen extra kosten.`,
      phoneNumber: {
        hint: `De SMS zal naar dit nummer worden verstuurd`,
        label: `telefoonnummer in internationaal formaat`
      },
      termsOfService: `Ik ga akkoord met de voorwaarden en condities hieronder.`
    }
  }
};
