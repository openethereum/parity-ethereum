import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { FlatButton } from 'material-ui';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';
import ContentSend from 'material-ui/svg-icons/content/send';
import NavigationArrowBack from 'material-ui/svg-icons/navigation/arrow-back';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import Api from '../../api';
import Modal from '../../ui/Modal';

import Complete from './Complete';
import Details from './Details';
import Extras from './Extras';
import ERRORS from './errors';

const DEFAULT_GAS = '21000';
const CONTRACT_GAS = '100000';
const DEFAULT_GASPRICE = '20000000000';
const TITLES = {
  transfer: 'transfer details',
  complete: 'complete',
  extras: 'extra information'
};
const STAGES_BASIC = [TITLES.transfer, TITLES.complete];
const STAGES_EXTRA = [TITLES.transfer, TITLES.extras, TITLES.complete];

export default class Transfer extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.object,
    onClose: PropTypes.func
  }

  state = {
    stage: 0,
    extraData: '',
    extraDataError: null,
    extras: false,
    gas: DEFAULT_GAS,
    gasEst: '0',
    gasError: null,
    gasPrice: DEFAULT_GASPRICE,
    gasPriceError: null,
    recipient: '',
    recipientError: ERRORS.requireRecipient,
    sending: false,
    tag: 'ΞTH',
    total: '0.0',
    totalError: null,
    value: '0.0',
    valueAll: false,
    valueError: null,
    isEth: true
  }

  componentDidMount () {
    this.getDefaults();
  }

  render () {
    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ this.state.stage }
        steps={ this.state.extras ? STAGES_EXTRA : STAGES_BASIC }
        visible>
        { this.renderPage() }
      </Modal>
    );
  }

  renderPage () {
    if (this.state.stage === 0) {
      return this.renderDetailsPage();
    } else if (this.state.stage === 1 && this.state.extras) {
      return this.renderExtrasPage();
    }

    return this.renderCompletePage();
  }

  renderCompletePage () {
    return (
      <Complete
        sending={ this.state.sending }
        txhash={ this.state.txhash } />
    );
  }

  renderDetailsPage () {
    return (
      <Details
        address={ this.props.account.address }
        all={ this.state.valueAll }
        extras={ this.state.extras }
        recipient={ this.state.recipient }
        recipientError={ this.state.recipientError }
        tag={ this.state.tag }
        total={ this.state.total }
        totalError={ this.state.totalError }
        value={ this.state.value }
        valueError={ this.state.valueError }
        onChange={ this.onUpdateDetails } />
    );
  }

  renderExtrasPage () {
    return (
      <Extras
        isEth={ this.state.isEth }
        extraData={ this.state.extraData }
        gas={ this.state.gas }
        gasEst={ this.state.gasEst }
        gasError={ this.state.gasError }
        gasPrice={ this.state.gasPrice }
        gasPriceError={ this.state.gasPriceError }
        total={ this.state.total }
        totalError={ this.state.totalError }
        onChange={ this.onUpdateDetails } />
    );
  }

  renderDialogActions () {
    const cancelBtn = (
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel' primary
        onTouchTap={ this.onClose } />
    );
    const nextBtn = (
      <FlatButton
        disabled={ !this.isValid() }
        icon={ <NavigationArrowForward /> }
        label='Next' primary
        onTouchTap={ this.onNext } />
    );
    const prevBtn = (
      <FlatButton
        icon={ <NavigationArrowBack /> }
        label='Back' primary
        onTouchTap={ this.onPrev } />
    );
    const sendBtn = (
      <FlatButton
        disabled={ !this.isValid() || this.state.sending }
        icon={ <ContentSend /> }
        label='Send' primary
        onTouchTap={ this.onSend } />
    );
    const doneBtn = (
      <FlatButton
        icon={ <ActionDoneAll /> }
        label='Close' primary
        onTouchTap={ this.onClose } />
    );

    switch (this.state.stage) {
      case 0:
        return this.state.extras
          ? [cancelBtn, nextBtn]
          : [cancelBtn, sendBtn];
      case 1:
        return this.state.extras
          ? [cancelBtn, prevBtn, sendBtn]
          : [doneBtn];
      default:
        return [doneBtn];
    }
  }

  isValid () {
    const detailsValid = !this.state.recipientError && !this.state.valueError && !this.state.totalError;
    const extrasValid = !this.state.gasError && !this.state.gasPriceError && !this.state.totalError;
    const verifyValid = !this.state.passwordError;

    switch (this.state.stage) {
      case 0:
        return detailsValid;

      case 1:
        return this.state.extras ? extrasValid : verifyValid;

      case 2:
        return verifyValid;
    }
  }

  onNext = () => {
    this.setState({
      stage: this.state.stage + 1
    });
  }

  onPrev = () => {
    this.setState({
      stage: this.state.stage - 1
    });
  }

  _onUpdateAll (valueAll) {
    this.setState({
      valueAll
    }, this.recalculate);
  }

  _onUpdateExtras (extras) {
    this.setState({
      extras
    });
  }

  _onUpdateExtraData (extraData) {
    this.setState({
      extraData
    });
  }

  validatePositiveNumber (num) {
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

  _onUpdateGas (gas) {
    const gasError = this.validatePositiveNumber(gas);

    this.setState({
      gas,
      gasError
    }, this.recalculate);
  }

  _onUpdateGasPrice (gasPrice) {
    const gasPriceError = this.validatePositiveNumber(gasPrice);

    this.setState({
      gasPrice,
      gasPriceError
    }, this.recalculate);
  }

  _onUpdateRecipient (recipient) {
    let recipientError = null;

    if (!recipient || !recipient.length) {
      recipientError = ERRORS.requireRecipient;
    } else if (!Api.format.isAddressValid(recipient)) {
      recipientError = ERRORS.invalidAddress;
    }

    this.setState({
      recipient,
      recipientError
    }, this.recalculateGas);
  }

  _onUpdateTag (tag) {
    this.setState({
      tag,
      isEth: tag === this.props.account.balances[0].token.tag
    }, this.recalculateGas);
  }

  _onUpdateValue (value) {
    const valueError = this.validatePositiveNumber(value);

    this.setState({
      value,
      valueError
    }, this.recalculateGas);
  }

  onUpdateDetails = (type, value) => {
    switch (type) {
      case 'all':
        return this._onUpdateAll(value);

      case 'extras':
        return this._onUpdateExtras(value);

      case 'extraData':
        return this._onUpdateExtraData(value);

      case 'gas':
        return this._onUpdateGas(value);

      case 'gasPrice':
        return this._onUpdateGasPrice(value);

      case 'recipient':
        return this._onUpdateRecipient(value);

      case 'tag':
        return this._onUpdateTag(value);

      case 'value':
        return this._onUpdateValue(value);
    }
  }

  _sendEth () {
    return this.context.api.eth
      .sendTransaction({
        from: this.props.account.address,
        to: this.state.recipient,
        gas: this.state.gas,
        gasPrice: this.state.gasPrice,
        value: Api.format.toWei(this.state.value)
      });
  }

  _sendToken () {
    const token = this.props.account.balances.find((balance) => balance.token.tag === this.state.tag).token;

    return token.contract.transfer
      .sendTransaction({
        from: this.props.account.address,
        to: token.address
      }, [
        this.state.recipient,
        new BigNumber(this.state.value).mul(token.format).toString()
      ]);
  }

  onSend = () => {
    this.onNext();
    this.setState({
      sending: true
    }, () => {
      (this.state.isEth
        ? this._sendEth()
        : this._sendToken()
      ).then((txhash) => {
        console.log('transaction', txhash);
        this.setState({
          sending: false,
          txhash: txhash
        });
      });
    });
  }

  onClose = () => {
    this.setState({
      stage: 0
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }

  _estimateGasToken () {
    const token = this.props.account.balances.find((balance) => balance.token.tag === this.state.tag).token;

    return token.contract.transfer
      .estimateGas({
        from: this.props.account.address,
        to: token.address
      }, [
        this.state.recipient,
        new BigNumber(this.state.value || 0).mul(token.format).toString()
      ]);
  }

  _estimateGasEth () {
    return this.context.api.eth
      .estimateGas({
        from: this.props.account.address,
        to: this.state.recipient,
        value: Api.format.toWei(this.state.value || 0)
      });
  }

  recalculateGas = () => {
    (this.state.isEth
      ? this._estimateGasEth()
      : this._estimateGasToken()
    ).then((_value) => {
      const extraGas = this.state.isEth ? 0 : CONTRACT_GAS;
      let gas = _value.add(extraGas);

      if (gas.add(extraGas).lt(DEFAULT_GAS)) {
        gas = new BigNumber(DEFAULT_GAS);
      }

      this.setState({
        gas: gas.toString(),
        gasEst: _value.toFormat()
      }, this.recalculate);
    });
  }

  recalculate = () => {
    if (!this.props.account) {
      return;
    }

    const gasTotal = new BigNumber(this.state.gasPrice || 0).mul(new BigNumber(this.state.gas || 0));
    const balances = this.props.account.balances;
    const balance = balances.find((balance) => this.state.tag === balance.token.tag);
    const availableEth = new BigNumber(balances[0].value);
    const available = new BigNumber(balance.value);
    const format = new BigNumber(balance.token.format || 1);

    let value = this.state.value;
    let valueError = this.state.valueError;
    let totalEth = gasTotal;
    let totalError = null;

    if (this.state.valueAll) {
      let bn;

      if (this.state.isEth) {
        bn = Api.format.fromWei(availableEth.minus(gasTotal));
      } else {
        bn = available.div(format);
      }

      value = (bn.lt(0) ? new BigNumber(0.0) : bn).toString();
    }

    if (this.state.isEth) {
      totalEth = totalEth.plus(Api.format.toWei(value || 0));
    }

    if (new BigNumber(value || 0).gt(available.div(format))) {
      valueError = ERRORS.largeAmount;
    } else if (valueError === ERRORS.largeAmount) {
      valueError = null;
    }

    if (totalEth.gt(availableEth)) {
      totalError = ERRORS.largeAmount;
    }

    this.setState({
      total: Api.format.fromWei(totalEth).toString(),
      totalError,
      value,
      valueError
    });
  }

  getDefaults = () => {
    const api = this.context.api;

    api.eth
      .gasPrice()
      .then((gasPrice) => {
        this.setState({
          gasPrice: gasPrice.toString()
        }, this.recalculate);
      });
  }
}
