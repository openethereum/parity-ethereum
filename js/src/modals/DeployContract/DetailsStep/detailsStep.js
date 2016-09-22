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

import { Form, Input, AddressSelect } from '../../../ui';

export default class DetailsStep extends Component {
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
    onAbiChange: PropTypes.func.isRequired,
    onCodeChange: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired,
    onDescriptionChange: PropTypes.func.isRequired,
    onNameChange: PropTypes.func.isRequired
  }

  render () {
    const { accounts } = this.props;
    const { abi, abiError, code, codeError, onAbiChange, onCodeChange, fromAddress, fromAddressError, name, nameError, onFromAddressChange, onNameChange } = this.props;

    return (
      <Form>
        <AddressSelect
          label='from account (contract owner)'
          hint='the owner account for this contract'
          value={ fromAddress }
          error={ fromAddressError }
          accounts={ accounts }
          onChange={ onFromAddressChange } />
        <Input
          label='contract name'
          hint='a name for the deployed contract'
          error={ nameError }
          value={ name }
          onSubmit={ onNameChange } />
        <Input
          label='abi'
          hint='the abi of the contract to deploy'
          error={ abiError }
          value={ abi }
          onSubmit={ onAbiChange } />
        <Input
          label='code'
          hint='the compiled code of the contract to deploy'
          error={ codeError }
          value={ code }
          onSubmit={ onCodeChange } />
      </Form>
    );
  }
}
