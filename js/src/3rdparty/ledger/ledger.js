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

import apiutil from '~/api/util';

import Ledger3 from './vendor/ledger3';
import LedgerEth from './vendor/ledger-eth';

const LEDGER_PATH = "44'/60'/0'";
const SCRAMBLE_KEY = 'w0w';

export default class Ledger {
  constructor (ledger) {
    this._ledger = ledger;
  }

  getAppConfiguration () {
    return new Promise((resolve, reject) => {
      this._ledger.getAppConfiguration((error, response) => {
        if (error) {
          reject(error);
          return;
        }

        resolve(response);
      });
    });
  }

  scan () {
    return new Promise((resolve, reject) => {
      this._ledger.getAddress(LEDGER_PATH, (error, response) => {
        if (error) {
          reject(error);
          return;
        }

        response.address = apiutil.toChecksumAddress(response.address);

        resolve(response);
      }, false, true);
    });
  }

  signTransaction (rawTransaction) {
    return new Promise((resolve, reject) => {
      this._ledger.signTransaction(LEDGER_PATH, rawTransaction, (error, response) => {
        if (error) {
          reject(error);
          return;
        }

        resolve(response);
      });
    });
  }

  static create (ledger) {
    if (!ledger) {
      ledger = new LedgerEth(new Ledger3(SCRAMBLE_KEY));
    }

    return new Ledger(ledger);
  }
}
