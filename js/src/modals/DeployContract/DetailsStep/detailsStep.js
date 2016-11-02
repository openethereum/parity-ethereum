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
import { validateAbi } from '../../../util/validation';

import styles from '../deployContract.css';

export default class DetailsStep extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    abi: PropTypes.string,
    abiError: PropTypes.string,
    code: PropTypes.string,
    codeError: PropTypes.string,
    description: PropTypes.string,
    descriptionError: PropTypes.string,
    fromAddress: PropTypes.string,
    fromAddressError: PropTypes.string,
    name: PropTypes.string,
    nameError: PropTypes.string,
    params: PropTypes.array,
    paramsError: PropTypes.array,
    onAbiChange: PropTypes.func.isRequired,
    onCodeChange: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired,
    onDescriptionChange: PropTypes.func.isRequired,
    onNameChange: PropTypes.func.isRequired,
    onParamsChange: PropTypes.func.isRequired
  }

  state = {
    inputs: []
  }

  render () {
    const { accounts } = this.props;
    const { abi, abiError, code, codeError, fromAddress, fromAddressError, name, nameError } = this.props;

    return (
      <Form>
        <AddressSelect
          label='from account (contract owner)'
          hint='the owner account for this contract'
          value={ fromAddress }
          error={ fromAddressError }
          accounts={ accounts }
          onChange={ this.onFromAddressChange } />
        <Input
          label='contract name'
          hint='a name for the deployed contract'
          error={ nameError }
          value={ name }
          onSubmit={ this.onNameChange } />
        <Input
          label='abi'
          hint='the abi of the contract to deploy'
          error={ abiError }
          value={ abi }
          onSubmit={ this.onAbiChange } />
        <Input
          label='code'
          hint='the compiled code of the contract to deploy'
          error={ codeError }
          value={ code }
          onSubmit={ this.onCodeChange } />
        { this.renderConstructorInputs() }
      </Form>
    );
  }

  renderConstructorInputs () {
    const { accounts, params, paramsError } = this.props;
    const { inputs } = this.state;

    if (!inputs || !inputs.length) {
      return null;
    }

    return inputs.map((input, index) => {
      const onChange = (event, value) => this.onParamChange(index, value);
      const onChangeBool = (event, _index, value) => this.onParamChange(index, value === 'true');
      const onSubmit = (value) => this.onParamChange(index, value);
      const label = `${input.name}: ${input.type}`;
      let inputBox = null;

      switch (input.type) {
        case 'address':
          inputBox = (
            <InputAddressSelect
              accounts={ accounts }
              editing
              label={ label }
              value={ params[index] }
              error={ paramsError[index] }
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
          inputBox = (
            <Select
              label={ label }
              value={ params[index] ? 'true' : 'false' }
              error={ paramsError[index] }
              onChange={ onChangeBool }>
              { boolitems }
            </Select>
          );
          break;

        default:
          inputBox = (
            <Input
              label={ label }
              value={ params[index] }
              error={ paramsError[index] }
              onSubmit={ onSubmit } />
            );
          break;
      }

      return (
        <div key={ index } className={ styles.funcparams }>
          { inputBox }
        </div>
      );
    });
  }

  onFromAddressChange = (event, fromAddress) => {
    const { onFromAddressChange } = this.props;

    onFromAddressChange(fromAddress);
  }

  onNameChange = (name) => {
    const { onNameChange } = this.props;

    onNameChange(name);
  }

  onParamChange = (index, value) => {
    const { params, onParamsChange } = this.props;

    params[index] = value;
    onParamsChange(params);
  }

  onAbiChange = (abi) => {
    const { api } = this.context;
    const { onAbiChange, onParamsChange } = this.props;
    const { abiError, abiParsed } = validateAbi(abi, api);

    if (!abiError) {
      const { inputs } = abiParsed.find((method) => method.type === 'constructor') || { inputs: [] };
      const params = [];

      inputs.forEach((input) => {
        switch (input.type) {
          case 'address':
            params.push('0x');
            break;

          case 'bool':
            params.push(false);
            break;

          case 'bytes':
            params.push('0x');
            break;

          case 'uint':
            params.push('0');
            break;

          case 'string':
            params.push('');
            break;

          default:
            params.push('0');
            break;
        }
      });

      onParamsChange(params);
      this.setState({ inputs });
    } else {
      onParamsChange([]);
      this.setState({ inputs: [] });
    }

    onAbiChange(abi);
  }

  onCodeChange = (code) => {
    const { onCodeChange } = this.props;

    onCodeChange(code);
  }
}
