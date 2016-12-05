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

import { observable, computed, action, transaction } from 'mobx';
import BigNumber from 'bignumber.js';

import ERRORS from './errors';
import { ERROR_CODES } from '../../api/transport/error';
import { DEFAULT_GAS, DEFAULT_GASPRICE, MAX_GAS_ESTIMATION } from '../../util/constants';

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
<<<<<<< HEAD
  @observable extras = false;
  @observable valueAll = false;
  @observable sending = false;
  @observable tag = 'ETH';
  @observable isEth = true;
  @observable busyState = null;
  @observable rejected = false;

  @observable data = '';
  @observable dataError = null;

  @observable gas = DEFAULT_GAS;
  @observable gasError = null;

  @observable gasEst = '0';
  @observable gasLimitError = null;
  @observable gasPrice = DEFAULT_GASPRICE;
  @observable gasPriceError = null;

  @observable recipient = '';
  @observable recipientError = ERRORS.requireRecipient;

  @observable sender = '';
  @observable senderError = null;

  @observable total = '0.0';
  @observable totalError = null;

  @observable value = '0.0';
  @observable valueError = null;
=======
  @observable data = '';
  @observable dataError = null;
  @observable extras = false;
  @observable gas = DEFAULT_GAS;
  @observable gasEst = '0';
  @observable gasError = null;
  @observable gasLimitError = null;
  @observable gasPrice = DEFAULT_GASPRICE;
  @observable gasPriceError = null;
  @observable recipient = '';
  @observable recipientError = ERRORS.requireRecipient;
  @observable sending = false;
  @observable tag = 'ETH';
  @observable total = '0.0';
  @observable totalError = null;
  @observable value = '0.0';
  @observable valueAll = false;
  @observable valueError = null;
  @observable isEth = true;
  @observable busyState = null;
  @observable rejected = false;
>>>>>>> master

  gasPriceHistogram = {};

  account = null;
  balance = null;
  gasLimit = null;
  onClose = null;

<<<<<<< HEAD
  isWallet = false;
  wallet = null;

=======
>>>>>>> master
  @computed get steps () {
    const steps = [].concat(this.extras ? STAGES_EXTRA : STAGES_BASIC);

    if (this.rejected) {
      steps[steps.length - 1] = TITLES.rejected;
    }

    return steps;
  }

  @computed get isValid () {
<<<<<<< HEAD
    const detailsValid = !this.recipientError && !this.valueError && !this.totalError && !this.senderError;
=======
    const detailsValid = !this.recipientError && !this.valueError && !this.totalError;
>>>>>>> master
    const extrasValid = !this.gasError && !this.gasPriceError && !this.totalError;
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

<<<<<<< HEAD
  get token () {
    return this.balance.tokens.find((balance) => balance.token.tag === this.tag).token;
  }

  constructor (api, props) {
    this.api = api;

    const { account, balance, gasLimit, senders, onClose } = props;
=======
  constructor (api, props) {
    this.api = api;

    const { account, balance, gasLimit, onClose } = props;
>>>>>>> master

    this.account = account;
    this.balance = balance;
    this.gasLimit = gasLimit;
    this.onClose = onClose;
<<<<<<< HEAD
    this.isWallet = account && account.wallet;

    if (this.isWallet) {
      this.wallet = props.wallet;
    }

    if (senders) {
      this.senderError = ERRORS.requireSender;
    }
=======
>>>>>>> master
  }

  @action onNext = () => {
    this.stage += 1;
  }

  @action onPrev = () => {
    this.stage -= 1;
  }

  @action onClose = () => {
    this.onClose && this.onClose();
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

<<<<<<< HEAD
      case 'sender':
        return this._onUpdateSender(value);

=======
>>>>>>> master
      case 'tag':
        return this._onUpdateTag(value);

      case 'value':
        return this._onUpdateValue(value);
    }
  }

  @action getDefaults = () => {
    Promise
      .all([
        this.api.parity.gasPriceHistogram(),
        this.api.eth.gasPrice()
      ])
      .then(([gasPriceHistogram, gasPrice]) => {
        transaction(() => {
          this.gasPrice = gasPrice.toString();
          this.gasPriceDefault = gasPrice.toFormat();
          this.gasPriceHistogram = gasPriceHistogram;

          this.recalculate();
        });
      })
      .catch((error) => {
        console.warn('getDefaults', error);
      });
  }

  @action onSend = () => {
    this.onNext();
    this.sending = true;

<<<<<<< HEAD
    this
      .send()
=======
    const promise = this.isEth ? this._sendEth() : this._sendToken();

    promise
>>>>>>> master
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
      })
      .catch((error) => {
        this.sending = false;
        this.newError(error);
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
    const gasError = this._validatePositiveNumber(gas);

    transaction(() => {
      this.gas = gas;
      this.gasError = gasError;

      this.recalculate();
    });
  }

  @action _onUpdateGasPrice = (gasPrice) => {
    const gasPriceError = this._validatePositiveNumber(gasPrice);

    transaction(() => {
      this.gasPrice = gasPrice;
      this.gasPriceError = gasPriceError;

      this.recalculate();
    });
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

<<<<<<< HEAD
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

=======
>>>>>>> master
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
      this.gas = 0;
      return this.recalculate();
    }

<<<<<<< HEAD
    this
      .estimateGas()
=======
    const promise = this.isEth ? this._estimateGasEth() : this._estimateGasToken();

    promise
>>>>>>> master
      .then((gasEst) => {
        let gas = gasEst;
        let gasLimitError = null;

        if (gas.gt(DEFAULT_GAS)) {
          gas = gas.mul(1.2);
        }

        if (gas.gte(MAX_GAS_ESTIMATION)) {
          gasLimitError = ERRORS.gasException;
        } else if (gas.gt(this.gasLimit)) {
          gasLimitError = ERRORS.gasBlockLimit;
        }

        transaction(() => {
          this.gas = gas.toFixed(0);
          this.gasEst = gasEst.toFormat();
          this.gasLimitError = gasLimitError;

          this.recalculate();
        });
      })
      .catch((error) => {
        console.error('etimateGas', error);
        this.recalculate();
      });
  }

  @action recalculate = () => {
    const { account, balance } = this;

    if (!account || !balance) {
      return;
    }

    const { gas, gasPrice, tag, valueAll, isEth } = this;

    const gasTotal = new BigNumber(gasPrice || 0).mul(new BigNumber(gas || 0));
    const balance_ = balance.tokens.find((b) => tag === b.token.tag);
    const availableEth = new BigNumber(balance.tokens[0].value);
    const available = new BigNumber(balance_.value);
    const format = new BigNumber(balance_.token.format || 1);

    let { value, valueError } = this;
    let totalEth = gasTotal;
    let totalError = null;

    if (valueAll) {
      if (isEth) {
        const bn = this.api.util.fromWei(availableEth.minus(gasTotal));
        value = (bn.lt(0) ? new BigNumber(0.0) : bn).toString();
      } else {
        value = available.div(format).toString();
      }
    }

    if (isEth) {
      totalEth = totalEth.plus(this.api.util.toWei(value || 0));
    }

    if (new BigNumber(value || 0).gt(available.div(format))) {
      valueError = ERRORS.largeAmount;
    } else if (valueError === ERRORS.largeAmount) {
      valueError = null;
    }

    if (totalEth.gt(availableEth)) {
      totalError = ERRORS.largeAmount;
    }

    transaction(() => {
      this.total = this.api.util.fromWei(totalEth).toString();
      this.totalError = totalError;
      this.value = value;
      this.valueError = valueError;
    });
  }

<<<<<<< HEAD
  send () {
    const { options, values } = this._getTransferParams();
    return this._getTransferMethod().postTransaction(options, values);
  }

  estimateGas () {
    const { options, values } = this._getTransferParams(true);
    return this._getTransferMethod(true).estimateGas(options, values);
  }

  _getTransferMethod (gas = false) {
    const { isEth, isWallet } = this;

    if (isEth && !isWallet) {
      return gas ? this.api.eth : this.api.parity;
    }

    if (isWallet) {
      return this.wallet.instance.execute;
    }

    return this.token.contract.instance.transfer;
  }

  _getTransferParams (gas = false) {
    const { isEth, isWallet } = this;

    const to = (isEth && !isWallet) ? this.recipient
      : (this.isWallet ? this.wallet.address : this.token.address);

    const options = {
      from: this.sender || this.account.address,
      to
    };

    if (!gas) {
      options.gas = this.gas;
      options.gasPrice = this.gasPrice;
    } else {
      options.gas = MAX_GAS_ESTIMATION;
    }

    if (isEth && !isWallet) {
      options.value = this.api.util.toWei(this.value || 0);

      if (this.data && this.data.length) {
        options.data = this.data;
      }

      return { options, values: [] };
    }

    const values = isWallet
      ? [
        this.recipient,
        this.api.util.toWei(this.value || 0),
        this.data || ''
      ]
      : [
        this.recipient,
        new BigNumber(this.value || 0).mul(this.token.format).toFixed(0)
      ];

    return { options, values };
=======
  _sendEth () {
    const { account, data, gas, gasPrice, recipient, value } = this;

    const options = {
      from: account.address,
      to: recipient,
      gas,
      gasPrice,
      value: this.api.util.toWei(value || 0)
    };

    if (data && data.length) {
      options.data = data;
    }

    return this.api.parity.postTransaction(options);
  }

  _sendToken () {
    const { account, balance } = this;
    const { gas, gasPrice, recipient, value, tag } = this;

    const token = balance.tokens.find((balance) => balance.token.tag === tag).token;

    return token.contract.instance.transfer
      .postTransaction({
        from: account.address,
        to: token.address,
        gas,
        gasPrice
      }, [
        recipient,
        new BigNumber(value).mul(token.format).toFixed(0)
      ]);
  }

  _estimateGasToken () {
    const { account, balance } = this;
    const { recipient, value, tag } = this;

    const token = balance.tokens.find((balance) => balance.token.tag === tag).token;

    return token.contract.instance.transfer
      .estimateGas({
        gas: MAX_GAS_ESTIMATION,
        from: account.address,
        to: token.address
      }, [
        recipient,
        new BigNumber(value || 0).mul(token.format).toFixed(0)
      ]);
  }

  _estimateGasEth () {
    const { account, data, recipient, value } = this;

    const options = {
      gas: MAX_GAS_ESTIMATION,
      from: account.address,
      to: recipient,
      value: this.api.util.toWei(value || 0)
    };

    if (data && data.length) {
      options.data = data;
    }

    return this.api.eth.estimateGas(options);
>>>>>>> master
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
