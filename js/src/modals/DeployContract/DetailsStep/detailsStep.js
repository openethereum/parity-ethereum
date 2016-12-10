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

import { AddressSelect, Form, Input, Select } from '~/ui';
import { validateAbi } from '~/util/validation';
import { parseAbiType } from '~/util/abi';

export default class DetailsStep extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,

    onFromAddressChange: PropTypes.func.isRequired,
    onNameChange: PropTypes.func.isRequired,
    onDescriptionChange: PropTypes.func.isRequired,
    onAbiChange: PropTypes.func.isRequired,
    onCodeChange: PropTypes.func.isRequired,
    onParamsChange: PropTypes.func.isRequired,
    onInputsChange: PropTypes.func.isRequired,

    balances: PropTypes.object,
    fromAddress: PropTypes.string,
    fromAddressError: PropTypes.string,
    name: PropTypes.string,
    nameError: PropTypes.string,
    description: PropTypes.string,
    descriptionError: PropTypes.string,
    abi: PropTypes.string,
    abiError: PropTypes.string,
    code: PropTypes.string,
    codeError: PropTypes.string,

    readOnly: PropTypes.bool
  };

  static defaultProps = {
    readOnly: false
  };

  state = {
    solcOutput: '',
    contracts: {},
    selectedContractIndex: 0
  }

  componentDidMount () {
    const { abi, code } = this.props;

    if (abi) {
      this.onAbiChange(abi);
      this.setState({ solcOutput: abi });
    }

    if (code) {
      this.onCodeChange(code);
    }
  }

  render () {
    const {
      accounts,
      balances,
      readOnly,

      fromAddress, fromAddressError,
      name, nameError,
      description, descriptionError,
      abiError,
      code, codeError
    } = this.props;

    const { solcOutput, contracts } = this.state;
    const solc = contracts && Object.keys(contracts).length > 0;

    return (
      <Form>
        <AddressSelect
          label='from account (contract owner)'
          hint='the owner account for this contract'
          value={ fromAddress }
          error={ fromAddressError }
          accounts={ accounts }
          balances={ balances }
          onChange={ this.onFromAddressChange } />

        <Input
          label='contract name'
          hint='a name for the deployed contract'
          error={ nameError }
          value={ name || '' }
          onChange={ this.onNameChange } />

        <Input
          label='contract description (optional)'
          hint='a description for the contract'
          error={ descriptionError }
          value={ description }
          onChange={ this.onDescriptionChange } />

        { this.renderContractSelect() }

        <Input
          label='abi / solc combined-output'
          hint='the abi of the contract to deploy or solc combined-output'
          error={ abiError }
          value={ solcOutput }
          onChange={ this.onSolcChange }
          onSubmit={ this.onSolcSubmit }
          readOnly={ readOnly } />
        <Input
          label='code'
          hint='the compiled code of the contract to deploy'
          error={ codeError }
          value={ code }
          onSubmit={ this.onCodeChange }
          readOnly={ readOnly || solc } />

      </Form>
    );
  }

  renderContractSelect () {
    const { contracts } = this.state;

    if (!contracts || Object.keys(contracts).length === 0) {
      return null;
    }

    const { selectedContractIndex } = this.state;
    const contractsItems = Object.keys(contracts).map((name, index) => (
      <MenuItem
        key={ index }
        label={ name }
        value={ index }
      >
        { name }
      </MenuItem>
    ));

    return (
      <Select
        label='select a contract'
        onChange={ this.onContractChange }
        value={ selectedContractIndex }
      >
        { contractsItems }
      </Select>
    );
  }

  onContractChange = (event, index) => {
    const { contracts } = this.state;
    const contractName = Object.keys(contracts)[index];
    const contract = contracts[contractName];

    if (!this.props.name || this.props.name.trim() === '') {
      this.onNameChange(null, contractName);
    }

    const { abi, bin } = contract;
    const code = /^0x/.test(bin) ? bin : `0x${bin}`;

    this.setState({ selectedContractIndex: index }, () => {
      this.onAbiChange(abi);
      this.onCodeChange(code);
    });
  }

  onSolcChange = (event, value) => {
    // Change triggered only if valid
    if (this.props.abiError) {
      return null;
    }

    this.onSolcSubmit(value);
  }

  onSolcSubmit = (value) => {
    try {
      const solcParsed = JSON.parse(value);

      if (!solcParsed || !solcParsed.contracts) {
        throw new Error('Wrong solc output');
      }

      this.setState({ contracts: solcParsed.contracts }, () => {
        this.onContractChange(null, 0);
      });
    } catch (e) {
      this.setState({ contracts: null });
      this.onAbiChange(value);
    }

    this.setState({ solcOutput: value });
  }

  onFromAddressChange = (event, fromAddress) => {
    const { onFromAddressChange } = this.props;

    onFromAddressChange(fromAddress);
  }

  onNameChange = (event, name) => {
    const { onNameChange } = this.props;

    onNameChange(name);
  }

  onDescriptionChange = (event, description) => {
    const { onDescriptionChange } = this.props;

    onDescriptionChange(description);
  }

  onAbiChange = (abi) => {
    const { api } = this.context;
    const { onAbiChange, onParamsChange, onInputsChange } = this.props;
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
      onInputsChange(inputs);
    } else {
      onParamsChange([]);
      onInputsChange([]);
    }

    onAbiChange(abi);
  }

  onCodeChange = (code) => {
    const { onCodeChange } = this.props;

    onCodeChange(code);
  }
}
