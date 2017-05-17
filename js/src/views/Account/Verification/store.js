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

import { observable, autorun, action } from 'mobx';

import Contract from '@parity/api/contract';
import { sha3 } from '@parity/api/util/sha3';
import Contracts from '@parity/shared/contracts';
import { checkIfVerified, findLastRequested, awaitPuzzle } from '@parity/shared/contracts/verification';
import { checkIfTxFailed, waitForConfirmations } from '@parity/shared/util/tx';

export const LOADING = 'fetching-contract';
export const QUERY_DATA = 'query-data';
export const POSTING_REQUEST = 'posting-request';
export const POSTED_REQUEST = 'posted-request';
export const REQUESTING_CODE = 'requesting-code';
export const QUERY_CODE = 'query-code';
export const POSTING_CONFIRMATION = 'posting-confirmation';
export const POSTED_CONFIRMATION = 'posted-confirmation';
export const DONE = 'done';

export default class VerificationStore {
  @observable step = null;
  @observable error = null;

  @observable contract = null;
  @observable fee = null;
  @observable accountIsVerified = null;
  @observable accountHasRequested = null;
  @observable isAbleToRequest = null;
  @observable lastRequestValues = null;
  @observable isServerRunning = null;
  @observable consentGiven = false;
  @observable requestTx = null;
  @observable code = '';
  @observable isCodeValid = false;
  @observable confirmationTx = null;

  constructor (api, abi, certifierName, account, isTestnet) {
    this._api = api;
    this.account = account;
    this.isTestnet = isTestnet;

    this.step = LOADING;
    Contracts.get(this._api).badgeReg.fetchCertifierByName(certifierName)
      .then(({ address }) => {
        this.contract = new Contract(api, abi).at(address);
        this.load();
      })
      .catch((err) => {
        console.error('error', err);
        this.error = 'Failed to fetch the contract: ' + err.message;
      });

    autorun(() => {
      if (this.error) {
        console.error('verification: ' + this.error);
      }
    });

    autorun(() => {
      if (this.step !== QUERY_DATA) {
        return;
      }

      this.setIfAbleToRequest();
    });
  }

  @action load = () => {
    const { contract, account } = this;

    this.step = LOADING;

    const isServerRunning = this.isServerRunning()
      .then((isRunning) => {
        this.isServerRunning = isRunning;
      })
      .catch((err) => {
        this.error = 'Failed to check if server is running: ' + err.message;
      });

    const fee = contract.instance.fee.call()
      .then((fee) => {
        this.fee = fee;
      })
      .catch((err) => {
        this.error = 'Failed to fetch the fee: ' + err.message;
      });

    const accountIsVerified = checkIfVerified(contract, account)
      .then((accountIsVerified) => {
        this.accountIsVerified = accountIsVerified;
      })
      .catch((err) => {
        this.error = 'Failed to check if verified: ' + err.message;
      });

    const accountHasRequested = findLastRequested(contract, account)
      .then((log) => {
        this.accountHasRequested = !!log;
        if (log) {
          this.lastRequestValues = log.params;
          this.requestTx = log.transactionHash;
        }
      })
      .catch((err) => {
        this.error = 'Failed to check if requested: ' + err.message;
      });

    Promise
      .all([ isServerRunning, fee, accountIsVerified, accountHasRequested ])
      .then(() => {
        this.step = QUERY_DATA;
      });
  }

  @action setConsentGiven = (consentGiven) => {
    this.consentGiven = consentGiven;
  }

  @action setCode = (code) => {
    const { contract, account } = this;

    if (!contract || !account || code.length === 0) {
      return;
    }

    const confirm = contract.functions.find((fn) => fn.name === 'confirm');
    const options = { from: account };
    const values = [ sha3.text(code) ];

    this.code = code;
    this.isCodeValid = false;
    confirm.estimateGas(options, values)
      .then((gas) => {
        options.gas = gas.mul(1.2).toFixed(0);
        return confirm.call(options, values);
      })
      .then((result) => {
        this.isCodeValid = result === true;
      })
      .catch((err) => {
        this.error = 'Failed to check if the code is valid: ' + err.message;
      });
  }

  requestValues = () => []

  @action sendRequest = () => {
    const { api, account, contract, fee } = this;

    const request = contract.functions.find((fn) => fn.name === 'request');
    const options = { from: account, value: fee.toString() };
    const values = this.requestValues();

    this.shallSkipRequest(values)
      .then((skipRequest) => {
        if (skipRequest) {
          return;
        }

        this.step = POSTING_REQUEST;
        return request.estimateGas(options, values)
          .then((gas) => {
            options.gas = gas.mul(1.2).toFixed(0);
            return request.postTransaction(options, values);
          })
          .then((handle) => {
            // The "request rejected" error doesn't have any property to distinguish
            // it from other errors, so we can't give a meaningful error here.
            return api.pollMethod('parity_checkRequest', handle);
          })
          .then((txHash) => {
            this.requestTx = txHash;
            return checkIfTxFailed(api, txHash, options.gas)
              .then((hasFailed) => {
                if (hasFailed) {
                  throw new Error('Transaction failed, all gas used up.');
                }
                this.step = POSTED_REQUEST;
                return waitForConfirmations(api, txHash, 1);
              });
          });
      })
      .then(() => this.checkIfReceivedCode())
      .then((hasReceived) => {
        if (hasReceived) {
          return;
        }

        this.step = REQUESTING_CODE;
        return this
          .requestCode()
          .then(() => awaitPuzzle(api, contract, account));
      })
      .then(() => {
        this.step = QUERY_CODE;
      })
      .catch((err) => {
        this.error = 'Failed to request a confirmation code: ' + err.message;
      });
  }

  @action queryCode = () => {
    this.step = QUERY_CODE;
  }

  @action sendConfirmation = () => {
    const { api, account, contract, code } = this;
    const token = sha3.text(code);

    const confirm = contract.functions.find((fn) => fn.name === 'confirm');
    const options = { from: account };
    const values = [ token ];

    this.step = POSTING_CONFIRMATION;
    confirm.estimateGas(options, values)
      .then((gas) => {
        options.gas = gas.mul(1.2).toFixed(0);
        return confirm.postTransaction(options, values);
      })
      .then((handle) => {
        // TODO: The "request rejected" error doesn't have any property to
        // distinguish it from other errors, so we can't give a meaningful error here.
        return api.pollMethod('parity_checkRequest', handle);
      })
      .then((txHash) => {
        this.confirmationTx = txHash;
        return checkIfTxFailed(api, txHash, options.gas)
          .then((hasFailed) => {
            if (hasFailed) {
              throw new Error('Transaction failed, all gas used up.');
            }
            this.step = POSTED_CONFIRMATION;
            return waitForConfirmations(api, txHash, 1);
          });
      })
      .then(() => {
        this.step = DONE;
      })
      .catch((err) => {
        this.error = 'Failed to send the verification code: ' + err.message;
      });
  }

  @action done = () => {
    this.step = DONE;
  }
}
