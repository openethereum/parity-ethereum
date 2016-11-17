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

import { AddressSelect, Form, Input, RadioButtons } from '../../../ui';

export default class DetailsStep extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    inputTypeValues: PropTypes.array.isRequired,

    onFromAddressChange: PropTypes.func.isRequired,
    onNameChange: PropTypes.func.isRequired,
    onInputTypeChange: PropTypes.func.isRequired,

    fromAddress: PropTypes.string,
    fromAddressError: PropTypes.string,
    name: PropTypes.string,
    nameError: PropTypes.string,
    inputType: PropTypes.object,
    readOnly: PropTypes.bool
  };

  static defaultProps = {
    readOnly: false
  };

  render () {
    const { accounts } = this.props;
    const { fromAddress, fromAddressError, name, nameError } = this.props;

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

        { this.renderChooseInputType() }
      </Form>
    );
  }

  renderChooseInputType () {
    const { readOnly } = this.props;

    if (readOnly) {
      return null;
    }

    const { inputTypeValues, inputType } = this.props;

    return (
      <div>
        <br />
        <p>Choose how ABI and Bytecode will be entered</p>
        <RadioButtons
          name='contractInputType'
          value={ inputType }
          values={ inputTypeValues }
          onChange={ this.onInputTypeChange }
        />
      </div>
    );
  }

  onFromAddressChange = (event, fromAddress) => {
    const { onFromAddressChange } = this.props;

    onFromAddressChange(fromAddress);
  }

  onNameChange = (name) => {
    const { onNameChange } = this.props;

    onNameChange(name);
  }

  onInputTypeChange = (inputType, index) => {
    const { onInputTypeChange } = this.props;
    onInputTypeChange(inputType, index);
  }
}
