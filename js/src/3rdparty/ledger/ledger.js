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

import 'u2f-api-polyfill';

import BigNumber from 'bignumber.js';
import Transaction from 'ethereumjs-tx';
import u2fapi from 'u2f-api';

import Ledger3 from './vendor/ledger3';
import LedgerEth from './vendor/ledger-eth';

const LEDGER_PATH_ETH = "44'/60'/0'/0";
const SCRAMBLE_KEY = 'w0w';

function numberToHex (number) {
  return `0x${new BigNumber(number).toString(16)}`;
}

export default class Ledger {
  constructor (api, ledger) {
    this._api = api;
    this._ledger = ledger;

    this._isSupported = false;

    this.checkJSSupport();
  }

  // FIXME: Until we have https support from Parity u2f will not work. Here we mark it completely
  // as unsupported until a full end-to-end environment is available.
  //
  // To test the implementation via http -
  //   - Install the u2f extension available from the Chrome store, https://chrome.google.com/webstore/detail/fido-u2f-universal-2nd-fa/pfboblefjcgdjicmnffhdgionmgcdmne/related
  //   - Navigate to chrome://extensions and enable Developer Mode by clicking a checkbox in the top right corner.
  //   - Find the FIDO U2F (Universal 2nd Factor) extension.
  //   - Click on "background page". This will open a Developer Tools window, including a Console.
  //   - In the console, type: HTTP_ORIGINS_ALLOWED = true;
  get isSupported () {
    return false && this._isSupported;
  }

  checkJSSupport () {
    return u2fapi
      .isSupported()
      .then((isSupported) => {
        console.log('Ledger:checkJSSupport', isSupported);

        this._isSupported = isSupported;
      });
  }

  getAppConfiguration () {
    return new Promise((resolve, reject) => {
      this._ledger.getAppConfiguration((response, error) => {
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
      this._ledger.getAddress(LEDGER_PATH_ETH, (response, error) => {
        if (error) {
          reject(error);
          return;
        }

        resolve([response.address]);
      }, true, false);
    });
  }

  signTransaction (transaction) {
    return this._api.net.version().then((_chainId) => {
      return new Promise((resolve, reject) => {
        const chainId = new BigNumber(_chainId).toNumber();
        const tx = new Transaction({
          data: transaction.data || transaction.input,
          gasPrice: numberToHex(transaction.gasPrice),
          gasLimit: numberToHex(transaction.gasLimit),
          nonce: numberToHex(transaction.nonce),
          to: transaction.to ? transaction.to.toLowerCase() : undefined,
          value: numberToHex(transaction.value),
          v: new Buffer([chainId]), // pass the chainId to the ledger
          r: new Buffer([]),
          s: new Buffer([])
        });
        const rawTransaction = tx.serialize().toString('hex');

        this._ledger.signTransaction(LEDGER_PATH_ETH, rawTransaction, (response, error) => {
          if (error) {
            reject(error);
            return;
          }

          const v = new Buffer(response.v, 'hex');

          if (chainId !== Math.floor((v[0] - 35) / 2)) {
            reject(new Error('Invalid EIP155 signature received from Ledger.'));
            return;
          }

          // https://github.com/ethcore/parity/pull/4578
          tx.v = new Buffer([1]); // new Buffer([(v[0] + 1) % 2]);
          tx.r = new Buffer(response.r, 'hex');
          tx.s = new Buffer(response.s, 'hex');

          resolve(`0x${tx.serialize().toString('hex')}`);
        });
      });
    });
  }

  static create (api, ledger) {
    if (!ledger) {
      ledger = new LedgerEth(new Ledger3(SCRAMBLE_KEY));
    }

    return new Ledger(api, ledger);
  }
}
