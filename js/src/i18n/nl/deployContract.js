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
  busy: {
    title: `Het contract wordt momenteel aangemaakt`
  },
  button: {
    cancel: `Annuleer`,
    close: `Sluit`,
    create: `Creëer`,
    done: `Klaar`,
    next: `Volgende`
  },
  completed: {
    description: `Je contract is aangemaakt en opgenomen in`
  },
  details: {
    abi: {
      hint: `de abi van het aan te maken contract of solc combined-output`,
      label: `abi / solc combined-output`
    },
    address: {
      hint: `het account wat eigenaar is van dit contract`,
      label: `van account (contract eigenaar)`
    },
    code: {
      hint: `de gecompileerde code van het aan te maken contract`,
      label: `code`
    },
    contract: {
      label: `selecteer een contract`
    },
    description: {
      hint: `een beschrijving van het contract`,
      label: `contract omschrijving (optioneel)`
    },
    name: {
      hint: `een naam voor het aangemaakte contract`,
      label: `contract naam`
    }
  },
  owner: {
    noneSelected: `er dient een geldig account als contract eigenaar geselecteerd te zijn`
  },
  parameters: {
    choose: `Kies de contract parameters`
  },
  rejected: {
    description: `Je kunt dit scherm veilig sluiten, het contract zal niet worden aangemaakt.`,
    title: `Het aanmaken van het contract is afgewezen`
  },
  state: {
    completed: `Het contract is succesvol aangemaakt`,
    preparing: `Transactie aan het voorbereiden om te verzenden op het netwerk`,
    validatingCode: `De contract code van het aangemaakte contract valideren`,
    waitReceipt: `Wachten tot het aanmaken van het contract bevestigd is`,
    waitSigner: `Wachten tot de transactie bevestigd is in de Parity Secure Signer`
  },
  title: {
    completed: `voltooid`,
    deployment: `aangemaakt`,
    details: `contract details`,
    failed: `aanmaken mislukt`,
    parameters: `contract parameters`,
    rejected: `afgewezen`
  }
};
