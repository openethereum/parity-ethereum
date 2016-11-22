// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
  label: 'Einstellungen',

  background: {
    label: 'Hintergrund',

    overview_0: 'Dein Hintergrundmuster ist einzigartig und beruht auf deiner Parity Installation. Es ändert sich jedes Mal dann, wenn du einen neuen Signer token erstellst. Dies stellt sicher, dass dezentrale Applicationen keine Vertrauenswürdigkeit vortäuschen können.',
    overview_1: 'Such dir ein Muster aus und merke es dir. Dieses Muster wird dir nun immer angezeigt, ausser du löschst deinen Browser Cache oder benutzt einen neuen Signer token.',

    button_more: 'weitere generieren'
  },

  parity: {
    label: 'Parity',

    overview_0: 'Diese Einstellungen verändern das Verhalten deines Parity-Knotens.',

    languages: {
      label: 'Anzeigesprache',
      hint: 'die Sprache, in der dir diese Obefläche angezeigt wird',

      language_en: 'English',
      language_de: 'Deutsch'
    },

    modes: {
      label: 'Betriebsmodus',
      hint: 'der Synchronisations-Modus deines Parity Knotens',

      mode_active: 'Parity synchronisiert kontinuierlich die Blockchain',
      mode_passive: 'Parity synchronisiert zunächst, schläft dann und wacht regelmäßig zum Synchronisieren auf',
      mode_dark: 'Parity synchronisiert nur, falls erforderlich - beim Aufruf einer fernen Prozedur (RPC)',
      mode_offline: 'Parity synchronisiert nicht'
    }
  },

  proxy: {
    label: 'Proxy',

    overview_0: 'Die Proxy-Einstellungen ermöglichen dir einfachen Zugriff auf die Parity-Oberfläche mit all ihren dezentralen Anwendungen über einprägsame Adressen.',

    details_0: 'Anstelle des Zugriffs über IP-Adresse und Port wirst du über die .parity Subdomain auf die Parity Oberfläche zugreifen können, indem du ',
    details_1: 'besuchst. Dafür musst du folgenden Eintrag in den Proxy-Einstellungen deines Browsers hinzufügen:',
    details_2: 'Hier findest du Anleitungen zum Anpassen der Proxy-Einstellungen in ',
    details_windows: 'Windows',
    details_3: ', ',
    details_macos: 'macOS',
    details_4: ' oder ',
    details_ubuntu: 'Ubuntu'
  },

  views: {
    label: 'Ansicht',

    overview_0: 'Hier kannst du einstellen, welche Teile der Parity-Oberfläche dir angezeigt werden sollen.',
    overview_1: 'Bist du Endnutzer? Die Standardeinstellungen sind gleichermaßen für Einsteiger als auch fortgeschrittene Nutzer gedacht.',
    overview_2: 'Bist du Entwickler? Füge z.B. den Verträge-Reiter zu deiner Ansicht hinzu.',
    overview_3: 'Bist du Miner oder betreibst du einen großangelegten Knoten? Füge den Status-Reiter hinzu, um alle Information über den Betrieb deines Knotens im Blick zu halten.',

    accounts: {
      label: 'Konten',
      description: 'Eine Liste aller Konten, die mit dieser Instanz von Parity verbunden sind. Sende Transaktionen, empfange eingehende Beträge, verwalte deinen Kontostand oder lade dein Konto auf.'
    },
    addresses: {
      label: 'Adressbuch',
      description: 'Eine Liste all deiner Kontakte und Adressbucheinträge, die von dieser Instanz von Parity verwaltet werden. Überwache Konten und gelange mit nur einem Klick zu Details deiner Transaktionen.'
    },
    apps: {
      label: 'Anwendungen',
      description: 'Dezentrale Anwendungen, die mit dem Netzwerk interagieren. Füge Anwendungen hinzu oder verwalte und interagiere mit bestehenden Anwendungen.'
    },
    contracts: {
      label: 'Verträge',
      description: 'Überwache und interagiere mit Verträgen, die im Netzwerk installiert wurden. Dies ist eine technisch fokussierte Umgebung, die auf fortgeschrittene Benutzer mit gutem Verständnis der Fuktionsweise von Verträgen zugeschnitten ist.'
    },
    status: {
      label: 'Status',
      description: 'Schau dir an, wie sich dein Parity Knoten schlägt. Hier findest du z.B. die Anzahl der aktuellen Verbindungen zum Netzwerk, Logs deiner laufenden Instanz und Mining Details (sofern eingeschaltet and konfiguriert).'
    },
    signer: {
      label: 'Signer',
      description: 'Dies ist der sichere Bereich zum Verwalten deiner Transaktionen. Hier kannst du Transaktionen, die von dir oder deinen Anwendungen angestoßen wurden, prüfen und dann genehmigen oder ablehnen.'
    },
    settings: {
      label: 'Einstellungen',
      description: 'Die aktuelle Seite, die dir erlaubt, die Parity Oberfläche nach deinen Wünschen anzupassen.'
    }
  }
};
