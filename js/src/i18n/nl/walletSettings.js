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
    title: `Eigenaar toevoegen`
  },
  buttons: {
    cancel: `Annuleer`,
    close: `Sluit`,
    next: `Volgende`,
    send: `Verzend`,
    sending: `Verzenden...`
  },
  changes: {
    modificationString: `Om je wijzigingen door te voeren zullen
              andere eigenaren deze zelfde wijzigingen moeten verzenden. Om het
              makkelijk te maken kunnen ze deze string kopieren-plakken:`,
    none: `Er zijn van deze Wallet geen instellingen gewijzigd.`,
    overview: `Je staat op het punt om de volgende wijzignen te maken`
  },
  edit: {
    message: `Om de instellingen van dit contract de wijzigen zullen
                  minimaal {owners, number} {owners, plural, one {owner } other {owners }} precies dezelfde
                  wijzigingen moeten verzenden. Je kunt de wijzigingen hier
                  in string-vorm plakken.`
  },
  modifications: {
    daylimit: {
      hint: `hoeveelheid uit te geven ETH zonder bevestiging met wachtwoord`,
      label: `wallet dag limiet`
    },
    fromString: {
      label: `wijzigingen`
    },
    owners: {
      label: `andere wallet eigenaren`
    },
    required: {
      hint: `vereiste aantal eigenaren om een transactie goed te keuren`,
      label: `vereiste eigenaren`
    },
    sender: {
      hint: `verzend wijzigingen als deze eigenaar`,
      label: `van account (wallet eigenaar)`
    }
  },
  ownersChange: {
    details: `van {from} naar {to}`,
    title: `Wijzig Vereiste Eigenaren`
  },
  rejected: `De transactie #{txid} is afgewezen`,
  removeOwner: {
    title: `Verwijder Eigenaar`
  }
};
