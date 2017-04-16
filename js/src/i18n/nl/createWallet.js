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
    add: `Voeg toe`,
    cancel: `Annuleer`,
    close: `Sluit`,
    create: `Creëer`,
    done: `Klaar`,
    next: `Volgende`,
    sending: `Verzenden...`
  },
  deployment: {
    message: `Het aanmaken wordt momenteel uitgevoerd`
  },
  details: {
    address: {
      hint: `het wallet contract adres`,
      label: `wallet adres`
    },
    dayLimitMulti: {
      hint: `hoeveelheid ETH die dagelijks kan worden uitgegeven zonder bevestigingen`,
      label: `wallet dag limiet`
    },
    description: {
      hint: `de lokale omschrijving voor dit wallet`,
      label: `wallet omschrijving (optioneel)`
    },
    descriptionMulti: {
      hint: `de lokale omschrijving voor dit wallet`,
      label: `wallet omschrijving (optioneel)`
    },
    name: {
      hint: `de lokale naam voor dit wallet`,
      label: `wallet naam`
    },
    nameMulti: {
      hint: `de lokale naam voor dit wallet`,
      label: `wallet naam`
    },
    ownerMulti: {
      hint: `het account wat eigenaar is van dit contract`,
      label: `van account (contract eigenaar)`
    },
    ownersMulti: {
      label: `andere wallet eigenaren`
    },
    ownersMultiReq: {
      hint: `vereiste aantal eigenaren om de transactie goed te keuren`,
      label: `vereiste eigenaren`
    }
  },
  info: {
    added: `toegevoegd`,
    copyAddress: `kopier adres naar klembord`,
    created: `{name} is {deployedOrAdded} in`,
    dayLimit: `De dag limiet is ingestel op {dayLimit} ETH.`,
    deployed: `aangemaakt`,
    numOwners: `{numOwners} eigenaren zijn vereist om de transactie goed te keuren.`,
    owners: `De wallet eigenaren zijn:`
  },
  rejected: {
    message: `Het aanmaken is mislukt`,
    state: `Je wallet zal niet worden aangemaakt. Je kunt dit venster nu veilig sluiten.`,
    title: `mislukt`
  },
  states: {
    completed: `Het contract is succesvol aangemaakt`,
    confirmationNeeded: `Voor het aanmaken van dit contract is bevestiging door andere eigenaren van het Wallet vereist`,
    preparing: `Transactie aan het voorbereiden voor verzending op het netwerk`,
    validatingCode: `De contract code van het aangemaakte contract wordt gevalideerd`,
    waitingConfirm: `Wachten tot de transactie bevestigd is in de Parity Secure Signer`,
    waitingReceipt: `Wachten tot het aanmaken van het contract bevestigd is`
  },
  steps: {
    deployment: `wallet aanmaken`,
    details: `wallet details`,
    info: `wallet informatie`,
    type: `wallet type`
  },
  type: {
    multisig: {
      description: `Maak een {link} Wallet aan`,
      label: `Multi-Sig wallet`,
      link: `standaard multi-signature`
    },
    watch: {
      description: `Voeg een bestaand wallet toe aan je accounts`,
      label: `Monitor/volg een wallet`
    }
  }
};
