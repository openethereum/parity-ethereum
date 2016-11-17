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

import React, { Component, PropTypes } from 'react';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { BusyStep, CompletedStep, Button, IdentityIcon, Modal, TxHash } from '../../ui';
import { validateAddress, validateUint } from '../../util/validation';

import DetailsStep from './DetailsStep';

import { ERROR_CODES } from '../../api/transport/error';

export default class ExecuteContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    isTest: PropTypes.bool,
    fromAddress: PropTypes.string,
    accounts: PropTypes.object,
    contract: PropTypes.object,
    onClose: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired
  }

  state = {
    amount: '0',
    amountError: null,
    fromAddressError: null,
    func: null,
    funcError: null,
    values: [],
    valuesError: [],
    step: 0,
    sending: false,
    busyState: null,
    txhash: null,
    rejected: false
  }

  componentDidMount () {
    const { contract } = this.props;
    const functions = contract.functions
      .filter((func) => !func.constant)
      .sort((a, b) => a.name.localeCompare(b.name));

    this.onFuncChange(null, functions[0]);
  }

  render () {
    const { sending } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        title='execute function'
        busy={ sending }
        waiting={ [1] }
        visible>
        { this.renderStep() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { onClose, fromAddress } = this.props;
    const { sending, step, fromAddressError, valuesError } = this.state;
    const hasError = fromAddressError || valuesError.find((error) => error);

    const cancelBtn = (
      <Button
        key='cancel'
        label='Cancel'
        icon={ <ContentClear /> }
        onClick={ onClose } />
    );

    if (step === 0) {
      return [
        cancelBtn,
        <Button
          key='postTransaction'
          label='post transaction'
          disabled={ sending || hasError }
          icon={ <IdentityIcon address={ fromAddress } button /> }
          onClick={ this.postTransaction } />
      ];
    } else if (step === 1) {
      return [
        cancelBtn
      ];
    }

    return [
      <Button
        key='close'
        label='Done'
        icon={ <ActionDoneAll /> }
        onClick={ onClose } />
    ];
  }

  renderStep () {
    const { onFromAddressChange } = this.props;
    const { step, busyState, txhash, rejected } = this.state;

    if (rejected) {
      return (
        <BusyStep
          title='The execution has been rejected'
          state='You can safely close this window, the function execution will not occur.'
        />
      );
    }

    if (step === 0) {
      return (
        <DetailsStep
          { ...this.props }
          { ...this.state }
          onAmountChange={ this.onAmountChange }
          onFromAddressChange={ onFromAddressChange }
          onFuncChange={ this.onFuncChange }
          onValueChange={ this.onValueChange } />
      );
    } else if (step === 1) {
      return (
        <BusyStep
          title='The function execution is in progress'
          state={ busyState } />
      );
    }

    return (
      <CompletedStep>
        <TxHash hash={ txhash } />
      </CompletedStep>
    );
  }

  onAmountChange = (amount) => {
    this.setState({ amount });
  }

  onFuncChange = (event, func) => {
    const values = func.inputs.map((input) => {
      switch (input.kind.type) {
        case 'address':
          return '0x';

        case 'bool':
          return false;

        case 'bytes':
          return '0x';

        case 'uint':
          return '0';

        default:
          return '';
      }
    });

    this.setState({
      func,
      values
    });
  }

  onValueChange = (event, index, _value) => {
    const { func, values, valuesError } = this.state;
    const input = func.inputs.find((input, _index) => index === _index);
    let value = _value;
    let valueError = null;

    switch (input.kind.type) {
      case 'address':
        valueError = validateAddress(_value).addressError;
        break;

      case 'bool':
        value = _value === 'true';
        break;

      case 'uint':
        valueError = validateUint(_value).valueError;
        break;
    }

    values[index] = value;
    valuesError[index] = valueError;

    this.setState({
      values: [].concat(values),
      valuesError: [].concat(valuesError)
    });
  }

  postTransaction = () => {
    const { api, store } = this.context;
    const { fromAddress } = this.props;
    const { amount, func, values } = this.state;
    const options = {
      from: fromAddress,
      value: api.util.toWei(amount || 0)
    };

    this.setState({ sending: true, step: 1 });

    func
      .estimateGas(options, values)
      .then((gas) => {
        options.gas = gas.mul(1.2).toFixed(0);
        console.log(`estimateGas: received ${gas.toFormat(0)}, adjusted to ${gas.mul(1.2).toFormat(0)}`);
        return func.postTransaction(options, values);
      })
      .then((requestId) => {
        this.setState({ busyState: 'Waiting for authorization in the Parity Signer' });

        return api
          .pollMethod('parity_checkRequest', requestId)
          .catch((e) => {
            if (e.code === ERROR_CODES.REQUEST_REJECTED) {
              this.setState({ rejected: true });
              return false;
            }

            throw e;
          });
      })
      .then((txhash) => {
        this.setState({ sending: false, step: 2, txhash, busyState: 'Your transaction has been posted to the network' });
      })
      .catch((error) => {
        console.error('postTransaction', error);
        store.dispatch({ type: 'newError', error });
      });
  }
}
