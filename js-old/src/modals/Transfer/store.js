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

import { noop } from 'lodash';
import { observable, computed, action, transaction } from 'mobx';
import BigNumber from 'bignumber.js';

import { eip20 as tokenAbi } from '~/contracts/abi';
import { fromWei } from '@parity/api/lib/util/wei';
import ERRORS from './errors';
import { DEFAULT_GAS } from '~/util/constants';
import { ETH_TOKEN } from '~/util/tokens';
import { getTxOptions } from '~/util/tx';
import GasPriceStore from '~/ui/GasPriceEditor/store';
import { getLogger, LOG_KEYS } from '~/config';

const log = getLogger(LOG_KEYS.TransferModalStore);

const TITLES = {
  transfer: 'transfer details',
  extras: 'extra information'
};
const STAGES_BASIC = [TITLES.transfer];
const STAGES_EXTRA = [TITLES.transfer, TITLES.extras];

export const WALLET_WARNING_SPENT_TODAY_LIMIT = 'WALLET_WARNING_SPENT_TODAY_LIMIT';

export default class TransferStore {
  @observable stage = 0;
  @observable extras = false;
  @observable isEth = true;
  @observable valueAll = false;
  @observable sending = false;
  @observable token = ETH_TOKEN;

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

  onClose = noop;
  senders = null;
  isWallet = false;
  tokenContract = null;
  tokens = {};
  wallet = null;

  gasStore = null;

  constructor (api, props) {
    this.api = api;

    const { account, balance, gasLimit, onClose, senders, newError, sendersBalances, tokens } = props;

    this.account = account;
    this.balance = balance;
    this.isWallet = account && account.wallet;
    this.newError = newError;
    this.tokens = tokens;

    this.gasStore = new GasPriceStore(api, { gasLimit });
    this.tokenContract = api.newContract(tokenAbi, '');

    if (this.isWallet) {
      this.wallet = props.wallet;
    }

    if (senders) {
      this.senders = senders;
      this.sendersBalances = sendersBalances;
      this.senderError = ERRORS.requireSender;
    }

    if (onClose) {
      this.onClose = onClose;
    }
  }

  @computed get steps () {
    const steps = [].concat(this.extras ? STAGES_EXTRA : STAGES_BASIC);

    return steps;
  }

  @computed get isValid () {
    const detailsValid = !this.recipientError && !this.valueError && !this.totalError && !this.senderError;
    const extrasValid = !this.gasStore.errorGas && !this.gasStore.errorPrice && !this.gasStore.conditionBlockError && !this.totalError;

    switch (this.stage) {
      case 0:
        return detailsValid;

      case 1:
        return extrasValid;
    }
  }

  @action onNext = () => {
    this.stage += 1;
  }

  @action onPrev = () => {
    this.stage -= 1;
  }

  @action handleClose = () => {
    this.onClose();
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

      case 'token':
        return this._onUpdateToken(value);

      case 'value':
        return this._onUpdateValue(value);
    }
  }

  @action onSend = () => {
    this.sending = true;

    this
      .send()
      .catch((error) => {
        this.newError(error);
      })
      .then(() => {
        this.handleClose();
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

  @action _onUpdateToken = (tokenId) => {
    transaction(() => {
      this.token = { ...this.tokens[tokenId] };
      this.isEth = this.token.native;

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

      // Don't show a warning if the limit is 0
      // (will always need confirmations)
      if (limit.gt(0)) {
        const remains = fromWei(limit.minus(spent));
        const today = Math.round(Date.now() / (24 * 3600 * 1000));
        const willResetLimit = last.lt(today);

        if ((!willResetLimit && remains.lt(value)) || fromWei(limit).lt(value)) {
          // already spent too much today
          this.walletWarning = WALLET_WARNING_SPENT_TODAY_LIMIT;
        } else if (this.walletWarning) {
          // all ok
          this.walletWarning = null;
        }
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

  /**
   * Return the balance of the selected token
   * (in WEI for ETH, without formating for other tokens)
   */
  getTokenBalance (token = this.token, address = this.account.address) {
    const balance = address === this.account.address
      ? this.balance
      : this.sendersBalances[address];

    return new BigNumber(balance[token.id] || 0);
  }

  getTokenValue (token = this.token, value = this.value, inverse = false) {
    let _value;

    try {
      _value = new BigNumber(value || 0);
    } catch (error) {
      _value = new BigNumber(0);
    }

    if (inverse) {
      return _value.div(token.format);
    }

    return _value.mul(token.format);
  }

  getValue () {
    const { valueAll, isEth, isWallet } = this;

    if (!valueAll) {
      const value = this.getTokenValue();

      return value;
    }

    const balance = this.getTokenBalance();

    if (!isEth || isWallet) {
      return balance;
    }

    // substract the gas estimate
    const gasTotal = new BigNumber(this.gasStore.price || 0)
      .mul(new BigNumber(this.gasStore.gas || 0));

    const totalEthValue = balance.gt(gasTotal)
      ? balance.minus(gasTotal)
      : new BigNumber(0);

    return totalEthValue;
  }

  getFormattedTokenValue (tokenValue) {
    return this.getTokenValue(this.token, tokenValue, true);
  }

  @action recalculate = (redo = false) => {
    const { account, balance } = this;

    if (!account || !balance) {
      return;
    }

    return this.getTransactionOptions()
      .then((options) => {
        const gasTotal = options.gas.mul(options.gasPrice);

        const tokenValue = this.getValue();
        const ethValue = options.value.add(gasTotal);

        const tokenBalance = this.getTokenBalance();
        const ethBalance = this.getTokenBalance(ETH_TOKEN, options.from);

        let totalError = null;
        let valueError = null;

        if (tokenValue.gt(tokenBalance)) {
          valueError = ERRORS.largeAmount;
        }

        if (ethValue.gt(ethBalance)) {
          totalError = ERRORS.largeAmount;
        }

        log.debug('@recalculate', {
          eth: ethValue.toFormat(),
          token: tokenValue.toFormat(),
          ethBalance: ethBalance.toFormat(),
          tokenBalance: tokenBalance.toFormat(),
          gasTotal: gasTotal.toFormat()
        });

        transaction(() => {
          this.totalError = totalError;
          this.valueError = valueError;
          this.gasStore.setErrorTotal(totalError);
          this.gasStore.setEthValue(options.value);

          this.total = fromWei(ethValue).toFixed();

          const nextValue = this.getFormattedTokenValue(tokenValue);
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
      });
  }

  estimateGas () {
    return this.getTransactionOptions()
      .then((options) => {
        return this.api.eth.estimateGas(options);
      });
  }

  send () {
    return this.getTransactionOptions()
      .then((options) => {
        log.debug('@send', 'transfer value', options.value && options.value.toFormat());

        return this.api.parity.postTransaction(options);
      });
  }

  getTransactionOptions () {
    const [ func, options, values ] = this._getTransactionArgs();

    return getTxOptions(this.api, func, options, values)
      .then((_options) => {
        delete _options.sender;
        return _options;
      });
  }

  _getTransactionArgs () {
    const { isEth } = this;

    const value = this.getValue();
    const options = this.gasStore.overrideTransaction({
      from: this.account.address,
      sender: this.sender
    });

    // A simple ETH transfer
    if (isEth) {
      options.value = value;
      options.data = this.data || '';
      options.to = this.recipient;

      return [ null, options ];
    }

    // A token transfer
    const tokenContract = this.tokenContract.at(this.token.address);
    const values = [
      this.recipient,
      value
    ];

    options.to = this.token.address;

    return [ tokenContract.instance.transfer, options, values ];
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
    const s = new BigNumber(num).mul(this.token.format || 1).toFixed();

    if (s.indexOf('.') !== -1) {
      return ERRORS.invalidDecimals;
    }

    return null;
  }
}
