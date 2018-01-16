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
    create: `Aanmaken`,
    done: `Klaar`,
    import: `Importeer`,
    next: `Volgende`,
    print: `Herstel zin afdrukken`
  },
  creationType: {
    fromGeth: {
      description: `Importeer accounts uit Geth keystore met het originele wachtwoord`,
      label: `Geth keystore`
    },
    fromJSON: {
      description: `Importeer account uit een JSON sleutelbestand met het originele wachtwoord`,
      label: `JSON bestand`
    },
    fromNew: {
      description: `Selecteer je identiteits-icoon en kies je wachtwoord`,
      label: `Nieuw Account`
    },
    fromPhrase: {
      description: `Herstel je account met een eerder bewaarde herstel zin en een nieuw wachtwoord`,
      label: `Herstel zin`
    },
    fromPresale: {
      description: `Importeer een Ethereum voor-verkoop (pre-sale) wallet bestand met het originele wachtwoord`,
      label: `voor-verkoop wallet`
    },
    fromRaw: {
      description: `Importeer een eerder gemaakte prive sleutel (raw private key) met een nieuw wachtwoord`,
      label: `Prive sleutel`
    },
    info: `Selecteer de manier waarop je je account wilt aanmaken of importeren. Maak een nieuw account aan met een naam en wachtwoord, of importeer/herstel een bestaand account vanuit verschillende bronnen zoals een herstel zin of een sleutelbestand. Met behulp van deze wizard word je door het proces begeleid om een account aan te maken.`
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
    available: `Er zijn momenteel {count} importeerbare sleutels (keys) beschikbaar vanuit Geth keystore, welke nog niet in je Parity installatie beschikbaar zijn. Selecteer de accounts die je wilt importeren en ga verder naar de volgende stap om het importeren te voltooien.`,
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
