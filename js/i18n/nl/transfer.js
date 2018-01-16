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
  advanced: {
    data: {
      hint: `de data om door te geven met de transactie`,
      label: `transactie data`
    }
  },
  buttons: {
    back: `Terug`,
    cancel: `Annuleer`,
    close: `Sluit`,
    next: `Volgende`,
    send: `Verzend`
  },
  details: {
    advanced: {
      label: `geavanceerde verzend opties`
    },
    amount: {
      hint: `de naar de ontvanger te verzenden hoeveelheid`,
      label: `te verzenden hoeveelheid (in {tag})`
    },
    fullBalance: {
      label: `volledige account balans`
    },
    recipient: {
      hint: `het ontvangende adres`,
      label: `ontvanger adres`
    },
    sender: {
      hint: `het verzendende adres`,
      label: `Verzender adres`
    },
    total: {
      label: `totale transactie hoeveelheid`
    }
  },
  wallet: {
    confirmation: `Deze transactie vereist bevestiging van andere eigenaren.`,
    operationHash: `hash van deze bewerking`
  },
  warning: {
    wallet_spent_limit: `De waarde van deze transactie is hoger dan de toegestane dag limiet en zal moeten worden bevestigd door andere eigenaren.`
  }
};
