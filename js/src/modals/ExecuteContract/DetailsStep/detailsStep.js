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

import React, { Component, PropTypes } from 'react';
import { Checkbox, MenuItem } from 'material-ui';

import { AddressSelect, Form, Input, Select, TypedInput } from '~/ui';
import { parseAbiType } from '~/util/abi';

import styles from '../executeContract.css';

const CHECK_STYLE = {
  position: 'absolute',
  top: '38px',
  left: '1em'
};

export default class DetailsStep extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    contract: PropTypes.object.isRequired,
    onAmountChange: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired,
    onValueChange: PropTypes.func.isRequired,
    values: PropTypes.array.isRequired,
    valuesError: PropTypes.array.isRequired,

    amount: PropTypes.string,
    amountError: PropTypes.string,
    balances: PropTypes.object,
    fromAddress: PropTypes.string,
    fromAddressError: PropTypes.string,
    func: PropTypes.object,
    funcError: PropTypes.string,
    gasEdit: PropTypes.bool,
    onFuncChange: PropTypes.func,
    onGasEditClick: PropTypes.func,
    warning: PropTypes.string
  }

  render () {
    const { accounts, amount, amountError, balances, fromAddress, fromAddressError, gasEdit, onGasEditClick, onFromAddressChange, onAmountChange } = this.props;

    return (
      <Form>
        { this.renderWarning() }
        <AddressSelect
          accounts={ accounts }
          balances={ balances }
          error={ fromAddressError }
          hint='the account to transact with'
          label='from account'
          onChange={ onFromAddressChange }
          value={ fromAddress } />
        { this.renderFunctionSelect() }
        { this.renderParameters() }
        <div className={ styles.columns }>
          <div>
            <Input
              error={ amountError }
              hint='the amount to send to with the transaction'
              label='transaction value (in ETH)'
              onSubmit={ onAmountChange }
              value={ amount } />
          </div>
          <div>
            <Checkbox
              checked={ gasEdit }
              label='edit gas price or value'
              onCheck={ onGasEditClick }
              style={ CHECK_STYLE } />
          </div>
        </div>
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
      .sort((a, b) => (a.name || '').localeCompare(b.name || ''))
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
      const onChange = (value) => onValueChange(null, index, value);
      const label = `${input.name}: ${input.type}`;

      return (
        <div
          key={ `${index}_${input.name || ''}` }
          className={ styles.funcparams }
        >
          <TypedInput
            label={ label }
            value={ values[index] }
            error={ valuesError[index] }
            onChange={ onChange }
            accounts={ accounts }
            param={ parseAbiType(input.type) }
          />
        </div>
      );
    });
  }

  renderWarning () {
    const { warning } = this.props;

    if (!warning) {
      return null;
    }

    return (
      <div className={ styles.warning }>
        { warning }
      </div>
    );
  }

  onFuncChange = (event, index, signature) => {
    const { contract, onFuncChange } = this.props;

    onFuncChange(event, contract.functions.find((fn) => fn.signature === signature));
  }
}
