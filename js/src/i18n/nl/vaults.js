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
  accounts: {
    button: {
      cancel: `Annuleer`,
      execute: `Stel in`
    },
    empty: `Er zitten geen accounts in deze kluis`,
    title: `Beheer Kluis Accounts`
  },
  button: {
    accounts: `accounts`,
    add: `Maak kluis`,
    close: `sluit`,
    edit: `bewerk`,
    open: `open`
  },
  confirmClose: {
    info: `Je staat op het punt op een kluis te sluiten. Alle aan deze kluis verbonden accounts zullen niet meer zichtbaar zijn na het voltooien van deze actie. Om deze accounts weer zichtbaar te maken dien je de kluis weer te openen.`,
    title: `Sluit Kluis`
  },
  confirmOpen: {
    info: `Je staat op het punt om een kluis te openen. Na de bevestiging met je wachtwoord zullen alle aan deze kluis verbonden account zichtbaar worden. Wanneer je de kluis weer sluit zullen deze accounts weer onzichtbaar worden, tot je de kluis weer opent.`,
    password: {
      hint: `het wachtwoord wat je hebt gekozen bij het aanmaken van de kluis`,
      label: `kluis wachtwoord`
    },
    title: `Open Kluis`
  },
  create: {
    button: {
      close: `sluit`,
      vault: `maak kluis`
    },
    description: {
      hint: `een uitgebereide omschrijving voor de kluis`
    },
    descriptions: {
      label: `(optioneel) omschrijving`
    },
    hint: {
      hint: `(optioneel) een hint om je het wachtwoord te helpen herinneren`,
      label: `wachtwoord hint`
    },
    name: {
      hint: `een naam voor de kluis`,
      label: `kluis naam`
    },
    password: {
      hint: `een sterk en uniek wachtwoord`,
      label: `wachtwoord`
    },
    password2: {
      hint: `verifieer je wachtwoord`,
      label: `wachtwoord (herhaal)`
    },
    title: `Maak een nieuwe kluis aan`
  },
  editMeta: {
    allowPassword: `Wijzig kluis wachtwoord`,
    button: {
      close: `sluit`,
      save: `opslaan`
    },
    currentPassword: {
      hint: `je huidige kluis wachtwoord`,
      label: `huidige wachtwoord`
    },
    description: {
      hint: `de  omschrijving van deze kluis`,
      label: `kluis omschrijving`
    },
    password: {
      hint: `een sterk, uniek wachtwoord`,
      label: `nieuw wachtwoord`
    },
    password2: {
      hint: `verifieer je nieuwe wachtwoord`,
      label: `nieuw wachtwoord (herhaal)`
    },
    passwordHint: {
      hint: `je wachtwoord hint voor deze kluis`,
      label: `wachtwoord hint`
    },
    title: `Bewerk Kluis Metadata`
  },
  empty: `Er zijn momenteel geen kluizen om weer tegeven.`,
  selector: {
    noneAvailable: `Er zijn momenteel geen kluizen geopend en beschikbaar voor selectie. Maak eerst een kluis aan en open deze, voordat je een kluis selecteert voor het verplaatsen van een account.`,
    title: `Selecteer Account Kluis`
  },
  title: `Kluis Beheer`
};
