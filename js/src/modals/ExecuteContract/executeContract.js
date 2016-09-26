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

import { Button, IdentityIcon, Modal } from '../../ui';

import CompletedStep from './CompletedStep';
import DetailsStep from './DetailsStep';

const steps = ['function execute', 'completed'];

export default class ExecuteContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    fromAddress: PropTypes.string,
    accounts: PropTypes.object,
    contract: PropTypes.object,
    onClose: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired
  }

  state = {
    amount: '0',
    amountError: null,
    func: null,
    funcError: null,
    values: [],
    valuesError: [],
    step: 0,
    sending: false
  }

  componentDidMount () {
    const { contract } = this.props;
    const functions = contract.functions
      .filter((func) => !func.constant)
      .sort((a, b) => a.name.localeCompare(b.name));

    this.onFuncChange(null, functions[0]);
  }

  render () {
    const { step } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ step }
        steps={ steps }
        waiting={ [1] }
        visible>
        { this.renderStep() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { onClose, fromAddress } = this.props;
    const { step, sending } = this.state;

    switch (step) {
      case 0:
        return [
          <Button
            key='cancel'
            label='Cancel'
            icon={ <ContentClear /> }
            onClick={ onClose } />,
          <Button
            key='postTransaction'
            label='post transaction'
            disabled={ sending }
            icon={ <IdentityIcon address={ fromAddress } button /> }
            onClick={ this.postTransaction } />
        ];

      case 1:
        return [
          <Button
            key='close'
            label='close'
            icon={ <ActionDoneAll /> }
            onClick={ onClose } />
        ];
    }
  }

  renderStep () {
    const { onFromAddressChange } = this.props;
    const { step } = this.state;

    switch (step) {
      case 0:
        return (
          <DetailsStep
            { ...this.props }
            { ...this.state }
            onFromAddressChange={ onFromAddressChange }
            onFuncChange={ this.onFuncChange }
            onValueChange={ this.onValueChange } />
        );
      case 1:
        return (
          <CompletedStep />
        );
    }
  }

  onAmountChange = (event, amount) => {
    this.setState({
      amount
    });
  }

  onFuncChange = (event, func) => {
    const values = func.inputs.map((input) => {
      switch (input.kind.type) {
        case 'address':
          return '0x';

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
    let value;
    let valueError;

    switch (input.kind.type) {
      default:
        value = _value;
        valueError = null;
        break;
    }

    values[index] = value;
    valuesError[index] = valueError;

    this.setState({
      values,
      valuesError
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

    this.setState({
      sending: true,
      step: 1
    });

    func
      .estimateGas(options, values)
      .then((gas) => {
        options.gas = gas.mul(1.2).toFixed(0);
        return func.postTransaction(options, values);
      })
      .catch((error) => {
        console.error('postTransaction', error);
        store.dispatch({ type: 'newError', error });
      });
  }
}
