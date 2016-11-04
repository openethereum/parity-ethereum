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
import { MenuItem } from 'material-ui';

import { AddressSelect, Form, Input, InputAddressSelect, Select } from '../../../ui';

import styles from '../executeContract.css';

export default class DetailsStep extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    contract: PropTypes.object.isRequired,
    amount: PropTypes.string,
    amountError: PropTypes.string,
    onAmountChange: PropTypes.func.isRequired,
    fromAddress: PropTypes.string,
    fromAddressError: PropTypes.string,
    onFromAddressChange: PropTypes.func.isRequired,
    func: PropTypes.object,
    funcError: PropTypes.string,
    onFuncChange: PropTypes.func,
    values: PropTypes.array.isRequired,
    valuesError: PropTypes.array.isRequired,
    onValueChange: PropTypes.func.isRequired
  }

  render () {
    const { accounts, amount, amountError, fromAddress, fromAddressError, onFromAddressChange, onAmountChange } = this.props;

    return (
      <Form>
        <AddressSelect
          label='from account'
          hint='the account to transact with'
          value={ fromAddress }
          error={ fromAddressError }
          accounts={ accounts }
          onChange={ onFromAddressChange } />
        { this.renderFunctionSelect() }
        { this.renderParameters() }
        <Input
          label='transaction value (in ETH)'
          hint='the amount to send to with the transaction'
          value={ amount }
          error={ amountError }
          onSubmit={ onAmountChange } />
      </Form>
    );
  }

  renderFunctionSelect () {
    const { func, funcError, contract } = this.props;

    if (!func) {
      return null;
    }

    const functions = contract.functions
      .filter((func) => !func.constant)
      .sort((a, b) => a.name.localeCompare(b.name))
      .map((func) => {
        const params = (func.abi.inputs || [])
          .map((input, index) => {
            return (
              <span key={ input.name }>
                <span>{ index ? ', ' : '' }</span>
                <span className={ styles.paramname }>{ input.name }: </span>
                <span>{ input.type }</span>
              </span>
            );
          });
        const name = (
          <div>
            <span>{ func.name }</span>
            <span className={ styles.paramname }>(</span>
            { params }
            <span className={ styles.paramname }>)</span>
          </div>
        );

        return (
          <MenuItem
            key={ func.signature }
            value={ func.signature }
            label={ func.name || '()' }>
            { name }
          </MenuItem>
        );
      });

    return (
      <Select
        label='function to execute'
        hint='the function to call on the contract'
        error={ funcError }
        onChange={ this.onFuncChange }
        value={ func.signature }>
        { functions }
      </Select>
    );
  }

  renderParameters () {
    const { accounts, func, values, valuesError, onValueChange } = this.props;

    if (!func) {
      return null;
    }

    return (func.abi.inputs || []).map((input, index) => {
      const onChange = (event, value) => onValueChange(event, index, value);
      const onSelect = (event, _index, value) => onValueChange(event, index, value);
      const onSubmit = (value) => onValueChange(null, index, value);
      const label = `${input.name}: ${input.type}`;
      let inputbox;

      switch (input.type) {
        case 'address':
          inputbox = (
            <InputAddressSelect
              accounts={ accounts }
              editing
              label={ label }
              value={ values[index] }
              error={ valuesError[index] }
              onChange={ onChange } />
          );
          break;

        case 'bool':
          const boolitems = ['false', 'true'].map((bool) => {
            return (
              <MenuItem
                key={ bool }
                value={ bool }
                label={ bool }>{ bool }</MenuItem>
            );
          });
          inputbox = (
            <Select
              label={ label }
              value={ values[index] ? 'true' : 'false' }
              error={ valuesError[index] }
              onChange={ onSelect }>{ boolitems }</Select>
          );
          break;

        default:
          inputbox = (
            <Input
              label={ label }
              value={ values[index] }
              error={ valuesError[index] }
              onSubmit={ onSubmit } />
          );
      }

      return (
        <div className={ styles.funcparams } key={ index }>
          { inputbox }
        </div>
      );
    });
  }

  onFuncChange = (event, index, signature) => {
    const { contract, onFuncChange } = this.props;

    onFuncChange(event, contract.functions.find((fn) => fn.signature === signature));
  }
}
