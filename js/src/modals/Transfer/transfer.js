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
    address: PropTypes.string.isRequired,
    balance: PropTypes.object,
    visible: PropTypes.bool.isRequired,
    onClose: PropTypes.func
  }

  state = {
    stage: 0,
    extraData: '',
    extraDataError: null,
    extras: false,
    gas: DEFAULT_GAS,
    gasError: null,
    gasPrice: DEFAULT_GASPRICE,
    gasPriceError: null,
    recipient: '',
    recipientError: ERRORS.requireRecipient,
    sending: false,
    total: '0.0',
    totalError: null,
    value: '0.0',
    valueAll: false,
    valueError: null
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
        visible={ this.props.visible }>
        { this.renderPage() }
      </Modal>
    );
  }

  renderPage () {
    switch (this.state.stage) {
      case 0:
        return (
          <Details
            all={ this.state.valueAll }
            extras={ this.state.extras }
            recipient={ this.state.recipient }
            recipientError={ this.state.recipientError }
            total={ this.state.total }
            totalError={ this.state.totalError }
            value={ this.state.value }
            valueError={ this.state.valueError }
            onChange={ this.onUpdateDetails } />
        );

      default:
        if (this.state.stage === 1 && this.state.extras) {
          return (
            <Extras
              extraData={ this.state.extraData }
              gas={ this.state.gas }
              gasError={ this.state.gasError }
              gasPrice={ this.state.gasPrice }
              gasPriceError={ this.state.gasPriceError }
              total={ this.state.total }
              totalError={ this.state.totalError }
              onChange={ this.onUpdateDetails } />
          );
        }

        return (
          <Complete
            sending={ this.state.sending }
            txhash={ this.state.txhash } />
        );
    }
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
    });
  }

  _onUpdateValue (value) {
    const valueError = this.validatePositiveNumber(value);

    this.setState({
      value,
      valueError
    }, this.recalculate);
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

      case 'value':
        return this._onUpdateValue(value);
    }
  }

  onSend = () => {
    this.onNext();

    this.setState({
      sending: true
    });

    this.context.api.eth
      .sendTransaction({
        from: this.props.address,
        to: this.state.recipient,
        gas: this.state.gas,
        gasPrice: this.state.gasPrice,
        value: Api.format.toWei(this.state.value)
      })
      .then((txhash) => {
        console.log('transaction', txhash);
        this.setState({
          sending: false,
          txhash: txhash
        });
      })
      .catch((error) => {
        console.error(error);
      });
  }

  onChangeDetails = (valid, { value, recipient, total, extras }) => {
    this.setState({
      value,
      extras,
      recipient,
      total
    });
  }

  onChangePassword = (valid, { password }) => {
    this.setState({
      password
    });
  }

  onClose = () => {
    this.setState({
      stage: 0
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }

  recalculate = () => {
    let value = this.state.value;
    const gas = new BigNumber(this.state.gasPrice || 0).mul(new BigNumber(this.state.gas || 0));
    const balance = new BigNumber(this.props.balance.value || 0);

    if (this.state.valueAll) {
      const bn = Api.format.fromWei(balance.minus(gas));
      value = bn.lt(0) ? '0.0' : bn.toString();
    }

    const amount = Api.format.toWei(value || 0);
    const total = amount.plus(gas);
    let totalError = null;

    if (total.gt(balance)) {
      totalError = ERRORS.largeAmount;
    }

    this.setState({
      total: Api.format.fromWei(total).toString(),
      totalError,
      value
    });
  }

  getDefaults = () => {
    const api = this.context.api;

    api.eth
      .gasPrice()
      .then((gasprice) => {
        this.setState({
          gasprice: gasprice.toString()
        }, this.recalculate);
      });
  }
}
