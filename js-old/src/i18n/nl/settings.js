// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
  background: {
    button_more: `genereer meer`,
    overview_0: `Het achtergrond patroon dat je nu kunt zien is uniek voor jouw Parity installatie. Het veranderd elke keer als je een nieuw Signer token genereerd. Op deze manier kunnen gedecentraliseerde applicaties niet doen alsof ze betrouwbaar zijn.`,
    overview_1: `Kies het patroon dat je wilt en onthoud het. Dit patroon wordt vanaf nu altijd getoond, tenzij je je browser cache wist of een nieuw Signer token genereerd.`,
    label: `achtergrond`
  },
  parity: {
    chains: {
      chain_classic: `Parity synchroniseert met het Ethereum Classic netwerk`,
      chain_dev: `Parity gebruikt een lokale ontwikkelaars chain`,
      chain_foundation: `Parity synchroniseert met het Ethereum netwerk wat door de Ethereum Foundation is uitgebracht`,
      chain_kovan: `Parity synchroniseert met het Kovan test netwerk`,
      chain_olympic: `Parity synchroniseert met het Olympic test netwerk`,
      chain_ropsten: `Parity synchroniseert met het Ropsten test netwerk`,
      cmorden_kovan: `Parity synchroniseert met het Morden (Classic) test netwerk`,
      hint: `de chain waarmee de Parity node synchroniseert`,
      label: `te synchroniseren chain/netwerk`
    },
    languages: {
      hint: `de taal waarin deze interface wordt weergegeven`,
      label: `Weergave taal`
    },
    loglevels: `Kies hoeveel details er in het logboek worden bijgehouden.`,
    modes: {
      hint: `de synchronisatie modus van de Parity node`,
      label: `Synchronisatie modus`,
      mode_active: `Parity synchroniseert de chain continu`,
      mode_dark: `Parity synchroniseert alleen als de RPC actief is`,
      mode_offline: `Parity synchroniseert niet`,
      mode_passive: `Parity synchroniseert in het begin. Daarna slaapt Parity en wordt regelmatig wakker voor synchronisatie`
    },
    overview_0: `Pas de Parity node instellingen aan en kies de manier van synchroniseren in dit menu.`,
    label: `parity`
  },
  proxy: {
    details_0: `In plaats van Parity te openen via het IP adres en poort-nummer, kun je toegang verkrijgen tot het .parity sub-domein door {homeProxy} te bezoeken. Om sub-domein gebaseerde routing in te stellen, dien je de proxy vermelding aan je browser proxy instellingen toe te voegen,`,
    details_1: `Om je te helpen met het configureren van je proxy, zijn er instructies beschikbaar voor {windowsLink}, {macOSLink} or {ubuntuLink}.`,
    details_macos: `macOS`,
    details_ubuntu: `Ubuntu`,
    details_windows: `Windows`,
    overview_0: `Met de proxy instellingen heb je de mogelijkheid om via een makkelijk te onthouden adres toegang te verkrijgen tot Parity en alle onderliggende decentrale applicaties.`,
    label: `proxy`
  },
  views: {
    accounts: {
      description: `Een overzicht van alle aan deze Parity installatie verbonden accounts, inclusief geimporteerde accounts. Verzend transacties, ontvang inkomende transacties, berheer je saldo en financier je accounts.`,
      label: `Accounts`
    },
    addresses: {
      description: `Een overzicht van alle door deze Parity installatie beheerde contacten en adresboek items. Monitor en volg accounts waarbij je transactie details met slechts een muisklik kunt weergeven.`,
      label: `Adresboek`
    },
    apps: {
      description: `Decentrale applicaties die gebruik maken van het onderliggende Ethereum netwerk. Voeg applicaties toe, beheer je applicatie portfolio en maak gebruik van applicaties op het wereldwijde netwerk.`,
      label: `Applicaties`
    },
    contracts: {
      description: `Monitor, volg en maak gebruik van specifieke contracten die op het netwerk zijn gezet. Dit is een meer technisch gerichte omgeving, voornamelijk bedoeld voor geavanceerde gebruikers die de werking van bepaalde contracten goed begrijpen.`,
      label: `Contracten`
    },
    overview_0: `Beheer de beschikbare weergaven van deze interface, en selecteer enkel de delen van de applicatie die voor jou van belang zijn.`,
    overview_1: `Ben je een eind gebruiker? De standaard instellingen zijn geschikt voor zowel beginners als gevorderde gebruikers.`,
    overview_2: `Ben je een ontwikkelaar? Voeg enkele functies toe om je contracten te beheren en gebruik te maken van gedecentraliseerde applicaties.`,
    overview_3: `Ben je een miner of draai je een grootschalige node? Voeg enkele functies toe om je alle informatie te geven die je nodig hebt om je node te monitoren.`,
    settings: {
      description: `Deze weergave. Hiermee kun je Parity aan passen in termen van opties, bediening en look en feel.`,
      label: `Instellingen`
    },
    signer: {
      description: `Het beveiligde transactie beheergebied van de applicatie waar je goedkeuring kunt verlenen aan elke uitgaande transactie die je hebt gemaakt met Parity evenals de transacties die in de wachtrij zijn geplaatst door decentrale applicaties.`,
      label: `Signer`
    },
    status: {
      description: `Volg hoe de Parity node zijn werk doet en je verbind met het netwerk en bekijk de logboeken van de momenteel draaiende node met mining details (indien geconfigureerd en ingeschakeld).`,
      label: `Status`
    },
    label: `weergaven`,
    home: {
      label: `Thuis`
    }
  },
  label: `instellingen`
};
