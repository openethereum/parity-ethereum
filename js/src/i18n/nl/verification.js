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
    cancel: `Annuleer`,
    done: `Klaar`,
    next: `Volgende`
  },
  code: {
    error: `ongeldige code`,
    hint: `Voer de ontvangen code in.`,
    label: `verificatie code`,
    sent: `De verificatie code is verstuurd naar {receiver}.`
  },
  confirmation: {
    authorise: `De verificatie code zal naar het contract worden verzonden. Gebruik de Parity Signer om dit goed te keuren.`,
    windowOpen: `Houd dit scherm open.`
  },
  done: {
    message: `Gefeliciteerd, je account is geverifieerd!`
  },
  email: {
    enterCode: `Voer de code in die je per e-email hebt ontvangen.`
  },
  gatherData: {
    email: {
      hint: `de code zal naar dit adres worden verstuurd`,
      label: `e-mail adres`
    },
    phoneNumber: {
      hint: `De SMS zal naar dit nummer worden verstuurd`,
      label: `telefoon nummer in internationaal formaat`
    }
  },
  gatherDate: {
    email: {
      error: `ongeldig e-mail adres`
    },
    phoneNumber: {
      error: `ongeldig telefoon nummer`
    }
  },
  loading: `Laden van verificatie data.`,
  request: {
    authorise: `Een verificatie verzoek zal naar het contract worden verzonden. Gebruik de Parity Signer om dit goed te keuren.`,
    requesting: `Een code aanvragen bij de Parity-server en wachten tot de puzzel in het contract opgenomen wordt.`,
    windowOpen: `Houd dit scherm open.`
  },
  sms: {
    enterCode: `Voer de code in die je per SMS hebt ontvangen.`
  },
  steps: {
    code: `Voer Code in`,
    completed: `Voltooi`,
    confirm: `Bevestig`,
    data: `Voer Data in`,
    method: `Methode`,
    request: `Verzoek`
  },
  title: `verifieer je account`,
  types: {
    email: {
      description: `De hash van het e-mail adres waarvan je bewijst dat het van jou is, zal worden opgeslagen in de blockchain.`,
      label: `E-mail Verificatie`
    },
    sms: {
      description: `Het zal in de blockchain worden vast gelegd dat jij in het bezit bent van een telefoon nummer (not <em>which</em>).`,
      label: `SMS Verificatie`
    }
  }
};
