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
  add: {
    builtin: {
      desc: `Experimentele applicaties, ontwikkeld door het Parity team om te demonstreren wat de dapp mogelijkheden, integratie en experimentele opties zijn; en om netwerkbreed client gedrag te controleren.`,
      label: `Applicaties gebundeld met Parity`
    },
    label: `zichtbare applicaties`,
    local: {
      desc: `Alle lokaal door de gebruiker geinstalleerde applicaties die toegang hebben tot de Parity client.`,
      label: `Lokaal beschikbare applicaties`
    },
    network: {
      desc: `Deze applicaties zijn niet bij Parity aangesloten, noch worden ze gepubliceerd door Parity. Alle applicaties blijven in beheer van hun eigen auteur. Zorg ervoor dat je snapt wat het doel van een applicatie is, voordat je ermee aan de slag gaat.`,
      label: `Applicaties op het wereldwijde netwerk`
    }
  },
  button: {
    edit: `bewerk`,
    permissions: `toestemming`
  },
  external: {
    accept: `Ik begrijp dat deze toepassingen niet bij Parity zijn aangesloten`,
    warning: `Deze applicaties zijn gepuliceerd door derde partijen welke niet verwant zijn aan Parity en zijn dus ook niet door Parity uitgebracht. Alle applicaties blijven in beheer van hun eigen auteur. Zorg ervoor dat je snapt wat het doel van een applicatie is voordat je ermee aan de slag gaat.`
  },
  label: `Gedecentraliseerde Applicaties`,
  permissions: {
    label: `zichtbare dapp accounts`
  }
};
