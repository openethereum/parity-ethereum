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
  label: 'Einstellungen',

  background: {
    label: 'Hintergrund',

    overview_0: 'Dein Hintergrundmuster ist einzigartig und beruht auf deiner Parity-Installation. Es ändert sich immer dann, wenn du ein neues Signer-Token erstellst. So können dezentrale Applicationen keine Vertrauenswürdigkeit vortäuschen.',
    overview_1: 'Such dir ein Muster aus und merke es dir. Dieses Muster wird dir von nun an immer angezeigt, es sei denn du löschst deinen Browser-Cache oder benutzt ein neues Signer-Token.',

    button_more: 'weitere generieren'
  },

  parity: {
    label: 'Parity',

    overview_0: 'Diese Einstellungen verändern das Verhalten deines Parity-Knotens.',

    languages: {
      label: 'Anzeigesprache',
      hint: 'Die Sprache, in der dir diese Obefläche angezeigt wird'
    },

    modes: {
      label: 'Betriebsmodus',
      hint: 'Der Synchronisations-Modus deines Parity-Knotens',

      mode_active: 'Parity synchronisiert kontinuierlich die Blockchain',
      mode_passive: 'Parity synchronisiert zu Beginn, schläft dann und wacht regelmäßig zum Synchronisieren auf',
      mode_dark: 'Parity synchronisiert nur falls erforderlich, etwa wenn es aus der Ferne aufgerufen wird',
      mode_offline: 'Parity synchronisiert nicht'
    }
  },

  proxy: {
    label: 'Proxy',

    overview_0: 'Die Proxy-Einstellungen ermöglichen dir einen einfachen Zugriff über einprägsame Adressen, auf die Oberfläche mit all ihren dezentralen Anwendungen.',

    details_0: 'Anstelle des Zugriffs über IP-Adresse und Port wirst du über die .parity-Subdomain auf die Parity-Oberfläche zugreifen können, indem du {homeProxy} besuchst. Dafür musst du folgenden Eintrag in den Proxy-Einstellungen deines Browsers hinzufügen:',
    details_1: 'Hier findest du Anleitungen zum Anpassen der Proxy-Einstellungen in {windowsLink}, {macOSLink} oder {ubuntuLink}.'
  },

  views: {
    label: 'Ansicht',

    overview_0: 'Hier kannst du entscheiden, welche Teile der Parity-Oberfläche dir angezeigt werden sollen.',
    overview_1: 'Benutzt du Parity ganz normal? Die Standardeinstellungen sind gleichermaßen für Einsteigende als auch für Fortgeschrittene gedacht.',
    overview_2: 'Entwickelst du mit Parity? Füge zum Beispiel den Reiter "Contracts" zu deiner Ansicht hinzu.',
    overview_3: 'Bist du Miner oder betreibst du einen großen Knoten? Füge den Reiter "Status" hinzu, um alle Information über den Betrieb deines Knotens im Blick zu halten.',

    accounts: {
      label: 'Konten',
      description: 'Eine Liste aller Konten, die mit dieser Parity-Installation verbunden sind. Sende Transaktionen, empfange eingehende Beträge, verwalte deine Kontostände oder lade deine Konten auf.'
    },
    addresses: {
      label: 'Adressbuch',
      description: 'Eine Liste all deiner Kontakte und der Adressen, die von dieser Parity-Installation verwaltet werden. Überwache Konten und gelange mit nur einem Klick zu Details deiner Transaktionen.'
    },
    apps: {
      label: 'Anwendungen',
      description: 'Dezentrale Anwendungen, die mit dem Netzwerk interagieren. Füge Anwendungen hinzu, entferne oder öffne Anwendungen.'
    },
    contracts: {
      label: 'Contracts',
      description: 'Interagiere mit Smart Contracts im Netzwerk. Diese Umgebung ist auf Fortgeschrittene mit gutem Verständnis der Fuktionsweise von Smart Contracts zugeschnitten.'
    },
    status: {
      label: 'Status',
      description: 'Schau dir an, wie sich dein Parity-Knoten schlägt. Hier findest du zum Beispiel die Anzahl der aktuellen Verbindungen zum Netzwerk, Logs deiner laufenden Instanz und (sofern konfiguriert) Details zum Mining.'
    },
    signer: {
      label: 'Signer',
      description: 'In diesem sicheren Bereich kannst du Transaktionen, die von dir oder von dezentralen Anwendungen erstellt wurden, prüfen und dann genehmigen oder ablehnen.'
    },
    settings: {
      label: 'Einstellungen',
      description: 'Diese Seite. Pass die Parity-Oberfläche nach deinen Wünschen an.'
    }
  }
};
