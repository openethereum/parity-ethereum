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

import { Form, Input, TypedInput, Select } from '../../../ui';
import { validateAbi } from '../../../util/validation';
import { parseAbiType } from '../../../util/abi';

import styles from '../deployContract.css';

export default class ParametersStep extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    inputType: PropTypes.object.isRequired,

    onAbiChange: PropTypes.func.isRequired,
    onCodeChange: PropTypes.func.isRequired,
    onParamsChange: PropTypes.func.isRequired,

    abi: PropTypes.string,
    abiError: PropTypes.string,
    code: PropTypes.string,
    codeError: PropTypes.string,
    params: PropTypes.array,
    paramsError: PropTypes.array,

    readOnly: PropTypes.bool
  };

  static defaultProps = {
    readOnly: false
  };

  state = {
    inputs: [],
    solcOutput: '',
    contracts: {},
    selectedContractIndex: 0
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
    const { abi, abiError, code, codeError, readOnly, inputType } = this.props;

    const manualInput = inputType.key === 'MANUAL';

    return (
      <Form>
        { this.renderFromSOLC() }
        <Input
          label='abi'
          hint='the abi of the contract to deploy'
          error={ abiError }
          value={ abi }
          onSubmit={ this.onAbiChange }
          readOnly={ readOnly || !manualInput } />
        <Input
          label='code'
          hint='the compiled code of the contract to deploy'
          error={ codeError }
          value={ code }
          onSubmit={ this.onCodeChange }
          readOnly={ readOnly || !manualInput } />

        { this.renderConstructorInputs() }
      </Form>
    );
  }

  renderFromSOLC () {
    const { inputType } = this.props;

    if (inputType.key !== 'SOLC') {
      return null;
    }

    const { solcOutput, contracts } = this.state;
    const error = contracts && Object.keys(contracts).length
      ? null
      : 'enter a valid solc output';

    return (
      <div>
        <p>To get <code>solc</code> output, you can use this command:</p>
        <code>solc --combined-json abi,bin Contract.sol | xclip -selection c</code>
        <Input
          label='solc output'
          hint='the output of solc'
          value={ solcOutput }
          error={ error }
          onChange={ this.onSolcChange }
        />
        { this.renderContractSelect() }
      </div>
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

  renderConstructorInputs () {
    const { accounts, params, paramsError } = this.props;
    const { inputs } = this.state;

    if (!inputs || !inputs.length) {
      return null;
    }

    const inputsComponents = inputs.map((input, index) => {
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

    return (
      <div>
        <p>Choose the contract parameters</p>
        { inputsComponents }
      </div>
    );
  }

  onContractChange = (event, index) => {
    const { contracts } = this.state;
    const contractName = Object.keys(contracts)[index];
    const contract = contracts[contractName];

    const { abi, bin } = contract;
    const code = /^0x/.test(bin) ? bin : `0x${bin}`;

    this.setState({ selectedContractIndex: index }, () => {
      this.onAbiChange(abi);
      this.onCodeChange(code);
    });
  }

  onSolcChange = (event, value) => {
    try {
      const solcParsed = JSON.parse(value);

      if (!solcParsed && !solcParsed.contracts) {
        throw new Error('Wrong solc output');
      }

      this.setState({ contracts: solcParsed.contracts }, () => {
        this.onContractChange(null, 0);
      });
    } catch (e) {
      this.setState({ contracts: null });
      this.onAbiChange('');
      this.onCodeChange('');
    }

    this.setState({ solcOutput: value });
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
