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
  awaitingDepositStep: {
    awaitingConfirmation: `Wachten tot bevestigd is dat je {typeSymbol} storting op het account van het wisselkantoor is aangekomen.`,
    awaitingDeposit: `{shapeshiftLink} is aan het wachten op {typeSymbol} storting. Verzend de valuta vanuit je {typeSymbol} netwerk client naar -`,
    minimumMaximum: `{minimum} minimum, {maximum} maximum`
  },
  awaitingExchangeStep: {
    awaitingCompletion: `Wachten op de voltooiing van het omwisselen van de valuta en op de overschrijving van de valuta naar je Parity account.`,
    receivedInfo: `{shapeshiftLink} heeft een storting ontvangen van -`
  },
  button: {
    cancel: `Annuleer`,
    done: `Sluit`,
    shift: `Wissel valuta om`
  },
  completedStep: {
    completed: `{shapeshiftLink} heeft het omwisselen van de valuta voltooid.`,
    parityFunds: `De saldo wijziging zal spoedig in je Parity client worden weergegeven.`
  },
  errorStep: {
    info: `Het omwisselen van de valuta via {shapeshiftLink} is mislukt door een fout bij het wisselkantoor. De ontvangen foutmelding van het wisselkantoor is als volgt:`
  },
  optionsStep: {
    noPairs: `Er is momenteel geen wisselkoers voor het valuta-paar beschikbaar om de transactie mee uit te voeren.`,
    returnAddr: {
      hint: `het retouradres voor wanneer het verzenden mislukt`,
      label: `(optioneel) {coinSymbol} retouradres`
    },
    terms: {
      label: `Ik begrijp dat ShapeShift.io een dienst is van een derde partij en dat bij gebruik van deze service de overdracht van informatie en/of financiele middelen volledig buiten het beheer van Parity vallen`
    },
    typeSelect: {
      hint: `het type crypto valuta om te wisselen`,
      label: `verzend naar account vanuit`
    }
  },
  price: {
    minMax: `({minimum} minimum, {maximum} maximum)`
  },
  title: {
    completed: `voltooid`,
    deposit: `wachten op storting`,
    details: `details`,
    error: `omwisselen mislukt`,
    exchange: `wachten op omwisselen`
  },
  warning: {
    noPrice: `Geen prijs gevonden voor het gekozen type`
  }
};
