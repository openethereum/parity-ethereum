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
  accountDetails: {
    address: {
      hint: `Het netwerk adres van het account`,
      label: `adres`
    },
    name: {
      hint: `Een beschrijvende naam van het account`,
      label: `account naam`
    },
    phrase: {
      hint: `De account herstel zin`,
      label: `Eigenaar's herstel zin (houd deze woorden veilig en prive want hiermee kun je volledige, ongelimiteerde toegang tot het account verkrijgen).`
    }
  },
  accountDetailsGeth: {
    imported: `Je hebt {number} adressen geïmporteerd uit de Geth keystore:`
  },
  button: {
    back: `Terug`,
    cancel: `Annuleer`,
    close: `Sluit`,
    create: `Aanmaken`,
    import: `Importeer`,
    next: `Volgende`,
    print: `Herstel zin afdrukken`
  },
  creationType: {
    fromGeth: {
      label: `Importeer accounts uit Geth keystore`
    },
    fromJSON: {
      label: `Importeer account uit een opgeslagen JSON file`
    },
    fromNew: {
      label: `Handmatig account aanmaken`
    },
    fromPhrase: {
      label: `Herstel account met een herstel zin`
    },
    fromPresale: {
      label: `Importeer account van een Ethereum voor-verkoop (pre-sale) wallet`
    },
    fromRaw: {
      label: `Importeer een prive sleutel (raw private key)`
    }
  },
  newAccount: {
    hint: {
      hint: `(optioneel) een hint om je te helpen het wachtwoord te herinneren`,
      label: `wachtwoord hint`
    },
    name: {
      hint: `een beschrijvende naam van het account`,
      label: `account naam`
    },
    password: {
      hint: `een sterk en uniek wachtwoord`,
      label: `wachtwoord`
    },
    password2: {
      hint: `bevestig je wachtwoord`,
      label: `wachtwoord (herhaal)`
    }
  },
  newGeth: {
    noKeys: `Er zijn momenteel geen importeerbare sleutels (keys) beschikbaar in de Geth keystore; of ze zijn al in je Parity installatie beschikbaar`
  },
  newImport: {
    file: {
      hint: `het te importeren wallet bestand`,
      label: `wallet bestand`
    },
    hint: {
      hint: `(optioneel) een hint om je te helpen het wachtwoord te herinneren`,
      label: `wachtwoord hint`
    },
    name: {
      hint: `een beschrijvende naam van het account`,
      label: `account naam`
    },
    password: {
      hint: `het wachtwoord om je wallet te openen`,
      label: `wachtwoord`
    }
  },
  rawKey: {
    hint: {
      hint: `(optioneel) een hint om je te helpen het wachtwoord te herinneren`,
      label: `wachtwoord hint`
    },
    name: {
      hint: `een beschrijvende naam van het account`,
      label: `account naam`
    },
    password: {
      hint: `een sterk en uniek wachtwoord`,
      label: `wachtwoord`
    },
    password2: {
      hint: `herhaal je wachtwoord ter controle`,
      label: `wachtwoord (herhaling)`
    },
    private: {
      hint: `de hexadecimaal gecodeerde prive sleutel (raw private key)`,
      label: `prive sleutel`
    }
  },
  recoveryPhrase: {
    hint: {
      hint: `(optioneel) een hint om je te helpen het wachtwoord te herinneren`,
      label: `wachtwoord hint`
    },
    name: {
      hint: `een beschrijvende naam van het account`,
      label: `account naam`
    },
    password: {
      hint: `een sterk en uniek wachtwoord`,
      label: `wachtwoord`
    },
    password2: {
      hint: `herhaal je wachtwoord ter controle`,
      label: `wachtwoord (herhaling)`
    },
    phrase: {
      hint: `de account herstel zin opgebouwd uit een aantal willekeurige woorden`,
      label: `account herstel zin`
    },
    windowsKey: {
      label: `Sleutel (key) is aangemaakt met Parity <1.4.5 op Windows`
    }
  },
  title: {
    accountInfo: `account informatie`,
    createAccount: `account aanmaken`,
    createType: `manier van aanmaken`,
    importWallet: `importeer wallet`
  }
};
