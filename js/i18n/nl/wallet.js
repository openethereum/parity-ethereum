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
  buttons: {
    edit: `bewerk`,
    forget: `vergeet`,
    settings: `instellingen`,
    transfer: `verzend`
  },
  confirmations: {
    buttons: {
      confirmAs: `Bevestig als...`,
      revokeAs: `Herroep als...`
    },
    none: `Er zijn momenteel geen transacties die op bevestiging wachten.`,
    tooltip: {
      confirmed: `Bevestigd door {number}/{required} eigenaren`
    }
  },
  details: {
    requiredOwners: `Dit wallet vereist ten minste {owners} voor de goedkeuring van elke actie (transactions, modifications).`,
    requiredOwnersNumber: `{number} {numberValue, plural, one {owner} other {owners}}`,
    spent: `{spent} is vandaag besteed, van de {limit} ingestelde daglimiet. De daglimiet is op {date} opnieuw ingesteld`,
    title: `Details`
  },
  title: `Wallet Beheer`,
  transactions: {
    none: `Er zijn geen verzonden transacties.`,
    title: `Transacties`
  }
};
