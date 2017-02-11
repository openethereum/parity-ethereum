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

import { observable, computed, action, transaction } from 'mobx';
import BigNumber from 'bignumber.js';
import { uniq } from 'lodash';

import { wallet as walletAbi } from '~/contracts/abi';
import { bytesToHex } from '~/api/util/format';
import { fromWei } from '~/api/util/wei';
import Contract from '~/api/contract';
import ERRORS from './errors';
import { ERROR_CODES } from '~/api/transport/error';
import { DEFAULT_GAS, DEFAULT_GASPRICE, MAX_GAS_ESTIMATION } from '~/util/constants';
import GasPriceStore from '~/ui/GasPriceEditor/store';
import { getLogger, LOG_KEYS } from '~/config';

const log = getLogger(LOG_KEYS.TransferModalStore);

const TITLES = {
  transfer: 'transfer details',
  sending: 'sending',
  complete: 'complete',
  extras: 'extra information',
  rejected: 'rejected'
};
const STAGES_BASIC = [TITLES.transfer, TITLES.sending, TITLES.complete];
const STAGES_EXTRA = [TITLES.transfer, TITLES.extras, TITLES.sending, TITLES.complete];

export const WALLET_WARNING_SPENT_TODAY_LIMIT = 'WALLET_WARNING_SPENT_TODAY_LIMIT';

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

  @observable walletWarning = null;

  account = null;
  balance = null;
  onClose = null;

  senders = null;
  isWallet = false;
  wallet = null;

  gasStore = null;

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

  @computed get steps () {
    const steps = [].concat(this.extras ? STAGES_EXTRA : STAGES_BASIC);

    if (this.rejected) {
      steps[steps.length - 1] = TITLES.rejected;
    }

    return steps;
  }

  @computed get isValid () {
    const detailsValid = !this.recipientError && !this.valueError && !this.totalError && !this.senderError;
    const extrasValid = !this.gasStore.errorGas && !this.gasStore.errorPrice && !this.gasStore.conditionBlockError && !this.totalError;
    const verifyValid = !this.passwordError;

    switch (this.stage) {
      case 0:
        return detailsValid;

      case 1:
        return this.extras
          ? extrasValid
          : verifyValid;

      case 2:
        return verifyValid;
    }
  }

  get token () {
    return this.balance.tokens.find((balance) => balance.token.tag === this.tag).token;
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
    if (!txhash || /^(0x)?0*$/.test(txhash)) {
      return;
    }

    let ethSubscriptionId = null;

    // Number of blocks left to look-up (unsub after 15 blocks if nothing)
    let nBlocksLeft = 15;

    return this.api.subscribe('eth_blockNumber', () => {
      this.api.eth
        .getTransactionReceipt(txhash)
        .then((tx) => {
          if (nBlocksLeft <= 0) {
            this.api.unsubscribe(ethSubscriptionId);
            ethSubscriptionId = null;
            return;
          }

          if (!tx) {
            nBlocksLeft--;
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
        })
        .catch(() => {
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

    if (this.isWallet && !valueError) {
      const { last, limit, spent } = this.wallet.dailylimit;
      const remains = fromWei(limit.minus(spent));
      const today = Math.round(Date.now() / (24 * 3600 * 1000));
      const isResetable = last.lt(today);

      if ((!isResetable && remains.lt(value)) || fromWei(limit).lt(value)) {
        // already spent too much today
        this.walletWarning = WALLET_WARNING_SPENT_TODAY_LIMIT;
      } else if (this.walletWarning) {
        // all ok
        this.walletWarning = null;
      }
    }

    transaction(() => {
      this.value = value;
      this.valueError = valueError;

      this.recalculateGas();
    });
  }

  @action recalculateGas = (redo = true) => {
    if (!this.isValid) {
      return this.recalculate(redo);
    }

    return this
      .estimateGas()
      .then((gasEst) => {
        let gas = gasEst;

        if (gas.gt(DEFAULT_GAS)) {
          gas = gas.mul(1.2);
        }

        transaction(() => {
          this.gasStore.setEstimated(gasEst.toFixed(0));
          this.gasStore.setGas(gas.toFixed(0));

          this.recalculate(redo);
        });
      })
      .catch((error) => {
        this.gasStore.setEstimatedError();
        console.warn('etimateGas', error);
        this.recalculate(redo);
      });
  }

  getBalance (forceSender = false) {
    if (this.isWallet && !forceSender) {
      return this.balance;
    }

    const balance = this.senders
      ? this.sendersBalances[this.sender]
      : this.balance;

    return balance;
  }

  getToken (tag = this.tag, forceSender = false) {
    const balance = this.getBalance(forceSender);

    if (!balance) {
      return null;
    }

    const _tag = tag.toLowerCase();
    const token = balance.tokens.find((b) => b.token.tag.toLowerCase() === _tag);

    return token;
  }

  /**
   * Return the balance of the selected token
   * (in WEI for ETH, without formating for other tokens)
   */
  getTokenBalance (tag = this.tag, forceSender = false) {
    const token = this.getToken(tag, forceSender);

    if (!token) {
      return new BigNumber(0);
    }

    const value = new BigNumber(token.value || 0);

    return value;
  }

  getTokenValue (tag = this.tag, value = this.value, inverse = false) {
    const token = this.getToken(tag);

    if (!token) {
      return new BigNumber(0);
    }

    const format = token.token
      ? new BigNumber(token.token.format || 1)
      : new BigNumber(1);

    let _value;

    try {
      _value = new BigNumber(value || 0);
    } catch (error) {
      _value = new BigNumber(0);
    }

    if (token.token && token.token.tag.toLowerCase() === 'eth') {
      if (inverse) {
        return this.api.util.fromWei(_value);
      }

      return this.api.util.toWei(_value);
    }

    if (inverse) {
      return _value.div(format);
    }

    return _value.mul(format);
  }

  getValues (_gasTotal) {
    const gasTotal = new BigNumber(_gasTotal || 0);
    const { valueAll, isEth, isWallet } = this;

    log.debug('@getValues', 'gas', gasTotal.toFormat());

    if (!valueAll) {
      const value = this.getTokenValue();

      // If it's a token or a wallet, eth is the estimated gas,
      // and value is the user input
      if (!isEth || isWallet) {
        return {
          eth: gasTotal,
          token: value
        };
      }

      // Otherwise, eth is the sum of the gas and the user input
      const totalEthValue = gasTotal.plus(value);

      return {
        eth: totalEthValue,
        token: value
      };
    }

    // If it's the total balance that needs to be sent, send the total balance
    // if it's not a proper ETH transfer
    if (!isEth || isWallet) {
      const tokenBalance = this.getTokenBalance();

      return {
        eth: gasTotal,
        token: tokenBalance
      };
    }

    // Otherwise, substract the gas estimate
    const availableEth = this.getTokenBalance('ETH');
    const totalEthValue = availableEth.gt(gasTotal)
      ? availableEth.minus(gasTotal)
      : new BigNumber(0);

    return {
      eth: totalEthValue.plus(gasTotal),
      token: totalEthValue
    };
  }

  getFormattedTokenValue (tokenValue) {
    const token = this.getToken();

    if (!token) {
      return new BigNumber(0);
    }

    const tag = token.token && token.token.tag || '';

    return this.getTokenValue(tag, tokenValue, true);
  }

  @action recalculate = (redo = false) => {
    const { account } = this;

    if (!account || !this.balance) {
      return;
    }

    const balance = this.getBalance();

    if (!balance) {
      return;
    }

    const gasTotal = new BigNumber(this.gasStore.price || 0).mul(new BigNumber(this.gasStore.gas || 0));

    const ethBalance = this.getTokenBalance('ETH', true);
    const tokenBalance = this.getTokenBalance();
    const { eth, token } = this.getValues(gasTotal);

    let totalEth = gasTotal;
    let totalError = null;
    let valueError = null;

    if (eth.gt(ethBalance)) {
      totalError = ERRORS.largeAmount;
    }

    if (token && token.gt(tokenBalance)) {
      valueError = ERRORS.largeAmount;
    }

    log.debug('@recalculate', {
      eth: eth.toFormat(),
      token: token.toFormat(),
      ethBalance: ethBalance.toFormat(),
      tokenBalance: tokenBalance.toFormat(),
      gasTotal: gasTotal.toFormat()
    });

    transaction(() => {
      this.totalError = totalError;
      this.valueError = valueError;
      this.gasStore.setErrorTotal(totalError);
      this.gasStore.setEthValue(totalEth);

      this.total = this.api.util.fromWei(eth).toFixed();

      const nextValue = this.getFormattedTokenValue(token);
      let prevValue;

      try {
        prevValue = new BigNumber(this.value || 0);
      } catch (error) {
        prevValue = new BigNumber(0);
      }

      // Change the input only if necessary
      if (!nextValue.eq(prevValue)) {
        this.value = nextValue.toString();
      }

      // Re Calculate gas once more to be sure
      if (redo) {
        return this.recalculateGas(false);
      }
    });
  }

  send () {
    const { options, values } = this._getTransferParams();

    log.debug('@send', 'transfer value', options.value && options.value.toFormat());

    return this._getTransferMethod().postTransaction(options, values);
  }

  _estimateGas (forceToken = false) {
    const { options, values } = this._getTransferParams(true, forceToken);

    return this._getTransferMethod(true, forceToken).estimateGas(options, values);
  }

  estimateGas () {
    return this._estimateGas();
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

    const options = this.gasStore.overrideTransaction({
      from: this.sender || this.account.address,
      to
    });

    if (gas) {
      options.gas = MAX_GAS_ESTIMATION;
    }

    const gasTotal = new BigNumber(options.gas || DEFAULT_GAS).mul(options.gasPrice || DEFAULT_GASPRICE);
    const { token } = this.getValues(gasTotal);

    if (isEth && !isWallet && !forceToken) {
      options.value = token;
      options.data = this._getData(gas);

      return { options, values: [] };
    }

    if (isWallet && !forceToken) {
      const to = isEth ? this.recipient : this.token.contract.address;
      const value = isEth ? token : new BigNumber(0);

      const values = [
        to, value,
        this._getData(gas)
      ];

      return { options, values };
    }

    const values = [
      this.recipient,
      token.toFixed(0)
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
