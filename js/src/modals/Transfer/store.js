// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { observable, computed, action, transaction } from 'mobx';
import BigNumber from 'bignumber.js';
import { uniq } from 'lodash';

import { wallet as walletAbi } from '~/contracts/abi';
import { bytesToHex } from '~/api/util/format';
import Contract from '~/api/contract';
import ERRORS from './errors';
import { ERROR_CODES } from '~/api/transport/error';
import { DEFAULT_GAS, MAX_GAS_ESTIMATION } from '~/util/constants';
import GasPriceStore from '~/ui/GasPriceEditor/store';

const TITLES = {
  transfer: 'transfer details',
  sending: 'sending',
  complete: 'complete',
  extras: 'extra information',
  rejected: 'rejected'
};
const STAGES_BASIC = [TITLES.transfer, TITLES.sending, TITLES.complete];
const STAGES_EXTRA = [TITLES.transfer, TITLES.extras, TITLES.sending, TITLES.complete];

export default class TransferStore {
  @observable stage = 0;
  @observable extras = false;
  @observable valueAll = false;
  @observable sending = false;
  @observable tag = 'ETH';
  @observable isEth = true;
  @observable busyState = null;
  @observable rejected = false;

  @observable data = '';
  @observable dataError = null;

  @observable recipient = '';
  @observable recipientError = ERRORS.requireRecipient;

  @observable sender = '';
  @observable senderError = null;
  @observable sendersBalances = {};

  @observable total = '0.0';
  @observable totalError = null;

  @observable value = '0.0';
  @observable valueError = null;

  account = null;
  balance = null;
  onClose = null;

  senders = null;
  isWallet = false;
  wallet = null;

  gasStore = null;

  @computed get steps () {
    const steps = [].concat(this.extras ? STAGES_EXTRA : STAGES_BASIC);

    if (this.rejected) {
      steps[steps.length - 1] = TITLES.rejected;
    }

    return steps;
  }

  @computed get isValid () {
    const detailsValid = !this.recipientError && !this.valueError && !this.totalError && !this.senderError;
    const extrasValid = !this.gasStore.errorGas && !this.gasStore.errorPrice && !this.totalError;
    const verifyValid = !this.passwordError;

    switch (this.stage) {
      case 0:
        return detailsValid;

      case 1:
        return this.extras ? extrasValid : verifyValid;

      case 2:
        return verifyValid;
    }
  }

  get token () {
    return this.balance.tokens.find((balance) => balance.token.tag === this.tag).token;
  }

  constructor (api, props) {
    this.api = api;

    const { account, balance, gasLimit, senders, newError, sendersBalances } = props;
    this.account = account;
    this.balance = balance;
    this.isWallet = account && account.wallet;
    this.newError = newError;

    this.gasStore = new GasPriceStore(api, { gasLimit });

    if (this.isWallet) {
      this.wallet = props.wallet;
      this.walletContract = new Contract(this.api, walletAbi);
    }

    if (senders) {
      this.senders = senders;
      this.sendersBalances = sendersBalances;
      this.senderError = ERRORS.requireSender;
    }
  }

  @action onNext = () => {
    this.stage += 1;
  }

  @action onPrev = () => {
    this.stage -= 1;
  }

  @action handleClose = () => {
    this.stage = 0;
  }

  @action onUpdateDetails = (type, value) => {
    switch (type) {
      case 'all':
        return this._onUpdateAll(value);

      case 'extras':
        return this._onUpdateExtras(value);

      case 'data':
        return this._onUpdateData(value);

      case 'gas':
        return this._onUpdateGas(value);

      case 'gasPrice':
        return this._onUpdateGasPrice(value);

      case 'recipient':
        return this._onUpdateRecipient(value);

      case 'sender':
        return this._onUpdateSender(value);

      case 'tag':
        return this._onUpdateTag(value);

      case 'value':
        return this._onUpdateValue(value);
    }
  }

  @action onSend = () => {
    this.onNext();
    this.sending = true;

    this
      .send()
      .then((requestId) => {
        this.busyState = 'Waiting for authorization in the Parity Signer';

        return this.api
          .pollMethod('parity_checkRequest', requestId)
          .catch((e) => {
            if (e.code === ERROR_CODES.REQUEST_REJECTED) {
              this.rejected = true;
              return false;
            }

            throw e;
          });
      })
      .then((txhash) => {
        transaction(() => {
          this.onNext();

          this.sending = false;
          this.txhash = txhash;
          this.busyState = 'Your transaction has been posted to the network';
        });

        if (this.isWallet) {
          return this._attachWalletOperation(txhash);
        }
      })
      .catch((error) => {
        this.sending = false;
        this.newError(error);
      });
  }

  @action _attachWalletOperation = (txhash) => {
    let ethSubscriptionId = null;

    return this.api.subscribe('eth_blockNumber', () => {
      this.api.eth
        .getTransactionReceipt(txhash)
        .then((tx) => {
          if (!tx) {
            return;
          }

          const logs = this.walletContract.parseEventLogs(tx.logs);
          const operations = uniq(logs
            .filter((log) => log && log.params && log.params.operation)
            .map((log) => bytesToHex(log.params.operation.value)));

          if (operations.length > 0) {
            this.operation = operations[0];
          }

          this.api.unsubscribe(ethSubscriptionId);
          ethSubscriptionId = null;
        });
    }).then((subId) => {
      ethSubscriptionId = subId;
    });
  }

  @action _onUpdateAll = (valueAll) => {
    this.valueAll = valueAll;
    this.recalculateGas();
  }

  @action _onUpdateExtras = (extras) => {
    this.extras = extras;
  }

  @action _onUpdateData = (data) => {
    this.data = data;
    this.recalculateGas();
  }

  @action _onUpdateGas = (gas) => {
    this.recalculate();
  }

  @action _onUpdateGasPrice = (gasPrice) => {
    this.recalculate();
  }

  @action _onUpdateRecipient = (recipient) => {
    let recipientError = null;

    if (!recipient || !recipient.length) {
      recipientError = ERRORS.requireRecipient;
    } else if (!this.api.util.isAddressValid(recipient)) {
      recipientError = ERRORS.invalidAddress;
    }

    transaction(() => {
      this.recipient = recipient;
      this.recipientError = recipientError;

      this.recalculateGas();
    });
  }

  @action _onUpdateSender = (sender) => {
    let senderError = null;

    if (!sender || !sender.length) {
      senderError = ERRORS.requireSender;
    } else if (!this.api.util.isAddressValid(sender)) {
      senderError = ERRORS.invalidAddress;
    }

    transaction(() => {
      this.sender = sender;
      this.senderError = senderError;

      this.recalculateGas();
    });
  }

  @action _onUpdateTag = (tag) => {
    transaction(() => {
      this.tag = tag;
      this.isEth = tag.toLowerCase().trim() === 'eth';

      this.recalculateGas();
    });
  }

  @action _onUpdateValue = (value) => {
    let valueError = this._validatePositiveNumber(value);

    if (!valueError) {
      valueError = this._validateDecimals(value);
    }

    transaction(() => {
      this.value = value;
      this.valueError = valueError;

      this.recalculateGas();
    });
  }

  @action recalculateGas = () => {
    if (!this.isValid) {
      this.gasStore.setGas('0');
      return this.recalculate();
    }

    this
      .estimateGas()
      .then((gasEst) => {
        let gas = gasEst;

        if (gas.gt(DEFAULT_GAS)) {
          gas = gas.mul(1.2);
        }

        transaction(() => {
          this.gasStore.setEstimated(gasEst.toFixed(0));
          this.gasStore.setGas(gas.toFixed(0));

          this.recalculate();
        });
      })
      .catch((error) => {
        console.warn('etimateGas', error);
        this.recalculate();
      });
  }

  @action recalculate = () => {
    const { account } = this;

    if (!account || !this.balance) {
      return;
    }

    const balance = this.senders
      ? this.sendersBalances[this.sender]
      : this.balance;

    if (!balance) {
      return;
    }

    const { tag, valueAll, isEth, isWallet } = this;

    const gasTotal = new BigNumber(this.gasStore.price || 0).mul(new BigNumber(this.gasStore.gas || 0));

    const availableEth = new BigNumber(balance.tokens[0].value);

    const senderBalance = this.balance.tokens.find((b) => tag === b.token.tag);
    const format = new BigNumber(senderBalance.token.format || 1);
    const available = isWallet
      ? this.api.util.fromWei(new BigNumber(senderBalance.value))
      : (new BigNumber(senderBalance.value)).div(format);

    let { value, valueError } = this;
    let totalEth = gasTotal;
    let totalError = null;

    if (valueAll) {
      if (isEth && !isWallet) {
        const bn = this.api.util.fromWei(availableEth.minus(gasTotal));
        value = (bn.lt(0) ? new BigNumber(0.0) : bn).toString();
      } else if (isEth) {
        value = (available.lt(0) ? new BigNumber(0.0) : available).toString();
      } else {
        value = available.toString();
      }
    }

    if (isEth && !isWallet) {
      totalEth = totalEth.plus(this.api.util.toWei(value || 0));
    }

    if (new BigNumber(value || 0).gt(available)) {
      valueError = ERRORS.largeAmount;
    } else if (valueError === ERRORS.largeAmount) {
      valueError = null;
    }

    if (totalEth.gt(availableEth)) {
      totalError = ERRORS.largeAmount;
    }

    transaction(() => {
      this.total = this.api.util.fromWei(totalEth).toFixed();
      this.totalError = totalError;
      this.value = value;
      this.valueError = valueError;
      this.gasStore.setErrorTotal(totalError);
      this.gasStore.setEthValue(totalEth);
    });
  }

  send () {
    const { options, values } = this._getTransferParams();
    return this._getTransferMethod().postTransaction(options, values);
  }

  _estimateGas (forceToken = false) {
    const { options, values } = this._getTransferParams(true, forceToken);
    return this._getTransferMethod(true, forceToken).estimateGas(options, values);
  }

  estimateGas () {
    if (this.isEth || !this.isWallet) {
      return this._estimateGas();
    }

    return Promise
      .all([
        this._estimateGas(true),
        this._estimateGas()
      ])
      .then((results) => results[0].plus(results[1]));
  }

  _getTransferMethod (gas = false, forceToken = false) {
    const { isEth, isWallet } = this;

    if (isEth && !isWallet && !forceToken) {
      return gas ? this.api.eth : this.api.parity;
    }

    if (isWallet && !forceToken) {
      return this.wallet.instance.execute;
    }

    return this.token.contract.instance.transfer;
  }

  _getData (gas = false) {
    const { isEth, isWallet } = this;

    if (!isWallet || isEth) {
      return this.data && this.data.length ? this.data : '';
    }

    const func = this._getTransferMethod(gas, true);
    const { options, values } = this._getTransferParams(gas, true);

    return this.token.contract.getCallData(func, options, values);
  }

  _getTransferParams (gas = false, forceToken = false) {
    const { isEth, isWallet } = this;

    const to = (isEth && !isWallet) ? this.recipient
      : (this.isWallet ? this.wallet.address : this.token.address);

    const options = {
      from: this.sender || this.account.address,
      to
    };

    if (!gas) {
      options.gas = this.gasStore.gas;
      options.gasPrice = this.gasStore.price;
    } else {
      options.gas = MAX_GAS_ESTIMATION;
    }

    if (isEth && !isWallet && !forceToken) {
      options.value = this.api.util.toWei(this.value || 0);
      options.data = this._getData(gas);

      return { options, values: [] };
    }

    if (isWallet && !forceToken) {
      const to = isEth ? this.recipient : this.token.contract.address;
      const value = isEth ? this.api.util.toWei(this.value || 0) : new BigNumber(0);

      const values = [
        to, value,
        this._getData(gas)
      ];

      return { options, values };
    }

    const values = [
      this.recipient,
      new BigNumber(this.value || 0).mul(this.token.format).toFixed(0)
    ];

    return { options, values };
  }

  _validatePositiveNumber (num) {
    try {
      const v = new BigNumber(num);
      if (v.lt(0)) {
        return ERRORS.invalidAmount;
      }
    } catch (e) {
      return ERRORS.invalidAmount;
    }

    return null;
  }

  _validateDecimals (num) {
    const { balance } = this;

    if (this.tag === 'ETH') {
      return null;
    }

    const token = balance.tokens.find((balance) => balance.token.tag === this.tag).token;
    const s = new BigNumber(num).mul(token.format || 1).toFixed();

    if (s.indexOf('.') !== -1) {
      return ERRORS.invalidDecimals;
    }

    return null;
  }
}
