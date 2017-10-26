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
  actionbar: {
    export: {
      button: {
        export: `exporteer`
      }
    },
    import: {
      button: {
        cancel: `Annuleer`,
        confirm: `Bevestig`,
        import: `importeer`
      },
      confirm: `Bevestig dat dit is wat je wilt importeren.`,
      error: `Er is een fout opgetreden: {errorText}`,
      step: {
        error: `fout`,
        select: `selecteer een bestand`,
        validate: `valideer`
      },
      title: `Importeer vanuit een bestand`
    },
    search: {
      hint: `Voer zoekopdracht in...`
    },
    sort: {
      sortBy: `Sorteer op {label}`,
      typeDefault: `Standaard`,
      typeEth: `Sorteer op ETH`,
      typeName: `Sorteer op naam`,
      typeTags: `Sorteer op tags`
    }
  },
  balance: {
    none: `Geen tegoeden gekoppeld aan dit account`
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
  copyToClipboard: {
    copied: `{data} is naar het klembord gekopierd`
  },
  errors: {
    close: `sluit`
  },
  fileSelect: {
    defaultLabel: `Sleep hier een bestand naartoe, of klik om een bestand te selecteren voor uploaden`
  },
  gasPriceSelector: {
    customTooltip: {
      transactions: `{number} {number, plural, one {transaction} other {transactions}} met een ingestelde gasprijs tussen de {minPrice} en {maxPrice}`
    }
  },
  identityName: {
    null: `NUL`,
    unnamed: `NAAMLOOS`
  },
  methodDecoding: {
    condition: {
      block: `, {historic, select, true {Submitted} false {Submission}} in blok {blockNumber}`,
      time: `, {historic, select, true {Submitted} false {Submission}} op {timestamp}`
    },
    deploy: {
      address: `Een contract aangemaakt op adres`,
      params: `met de volgende parameters:`,
      willDeploy: `Zal een contract aanmaken`,
      withValue: `, verzenden van {value}`
    },
    gasUsed: `({gas} gas gebruikt)`,
    gasValues: `{gas} gas ({gasPrice}M/{tag})`,
    input: {
      data: `data`,
      input: `input`,
      withInput: `met de {inputDesc} {inputValue}`
    },
    receive: {
      contract: `het contract`,
      info: `{historic, select, true {Received} false {Will receive}} {valueEth} van {aContract}{address}`
    },
    signature: {
      info: `{historic, select, true {Executed} false {Will execute}} the {method} function on the contract {address} transferring {ethValue}{inputLength, plural, zero {,} other {passing the following {inputLength, plural, one {parameter} other {parameters}}}}`
    },
    token: {
      transfer: `{historic, select, true {Transferred} false {Will transfer}} {value} naar {address}`
    },
    transfer: {
      contract: `het contract`,
      info: `{historic, select, true {Transferred} false {Will transfer}} {valueEth} naar {aContract}{address}`
    },
    txValues: `{historic, select, true {Provided} false {Provides}} {gasProvided}{gasUsed} voor een totale transactie waarde van {totalEthValue}`,
    unknown: {
      info: `{historic, select, true {Executed} false {Will execute}} the {method} on the contract {address} transferring {ethValue}.`
    }
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
  vaultSelect: {
    hint: `de kluis waaraan dit account gekoppeld is`,
    label: `gekoppelde kluis`
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
      termsOfService: `Ik ga akkoord met de voorwaarden en condities hieronder.`
    }
  }
};
