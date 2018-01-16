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
  condition: {
    block: {
      hint: `Het minimum blok voor het verzenden`,
      label: `Transactie verzend blok`
    },
    blocknumber: `Verzend na bloknummer`,
    date: {
      hint: `De minimale datum voor het verzenden`,
      label: `Transactie verzend datum`
    },
    datetime: `Verzend na datum & tijdstip`,
    label: `Conditie waarbij transactie activeert`,
    none: `Geen condities`,
    time: {
      hint: `Het minimale tijdstip voor het verzenden`,
      label: `Transactie verzend tijdstip`
    }
  },
  gas: {
    info: `Je kunt de gas prijs kiezen op basis van de gas prijs van de transacties die recentelijk in de blokken werden opgenomen. Een lagere gas prijs betekend een goedkopere transactie. Een hogere gas prijs betekend dat je transactie sneller in een blok wordt opgenomen.`
  }
};
