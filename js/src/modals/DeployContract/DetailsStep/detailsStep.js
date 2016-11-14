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

import { AddressSelect, Form, Input, TypedInput } from '../../../ui';
import { validateAbi } from '../../../util/validation';
import { parseAbiType } from '../../../util/abi';

import styles from '../deployContract.css';

export default class DetailsStep extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

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
    onParamsChange: PropTypes.func.isRequired,
    readOnly: PropTypes.bool
  };

  static defaultProps = {
    readOnly: false
  };

  state = {
    inputs: []
  }

  componentDidMount () {
    const { abi, code } = this.props;

    if (abi) {
      this.onAbiChange(abi);
    }

    if (code) {
      this.onCodeChange(code);
    }
  }

  render () {
    const { accounts } = this.props;
    const { abi, abiError, code, codeError, fromAddress, fromAddressError, name, nameError, readOnly } = this.props;

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
          onSubmit={ this.onAbiChange }
          readOnly={ readOnly } />
        <Input
          label='code'
          hint='the compiled code of the contract to deploy'
          error={ codeError }
          value={ code }
          onSubmit={ this.onCodeChange }
          readOnly={ readOnly } />

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
      const onChange = (value) => this.onParamChange(index, value);

      const label = `${input.name ? `${input.name}: ` : ''}${input.type}`;
      const value = params[index];
      const error = paramsError[index];
      const param = parseAbiType(input.type);

      return (
        <div key={ index } className={ styles.funcparams }>
          <TypedInput
            label={ label }
            value={ value }
            error={ error }
            accounts={ accounts }
            onChange={ onChange }
            param={ param }
          />
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
      const { inputs } = abiParsed
        .find((method) => method.type === 'constructor') || { inputs: [] };

      const params = [];

      inputs.forEach((input) => {
        const param = parseAbiType(input.type);
        params.push(param.default);
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
