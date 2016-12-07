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

import { Form, TypedInput, Input, AddressSelect } from '../../../ui';
import { parseAbiType } from '../../../util/abi';

export default class WalletDetails extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    wallet: PropTypes.object.isRequired,
    errors: PropTypes.object.isRequired,
    onChange: PropTypes.func.isRequired
  };

  render () {
    const { accounts, wallet, errors } = this.props;

    return (
      <Form>
        <AddressSelect
          label='from account (contract owner)'
          hint='the owner account for this contract'
          value={ wallet.account }
          error={ errors.account }
          onChange={ this.onAccoutChange }
          accounts={ accounts }
        />

        <Input
          label='wallet name'
          hint='the local name for this wallet'
          value={ wallet.name }
          error={ errors.name }
          onChange={ this.onNameChange }
        />

        <Input
          label='wallet description (optional)'
          hint='the local description for this wallet'
          value={ wallet.description }
          onChange={ this.onDescriptionChange }
        />

        <TypedInput
          label='other wallet owners'
          value={ wallet.owners.slice() }
          onChange={ this.onOwnersChange }
          accounts={ accounts }
          param={ parseAbiType('address[]') }
        />

        <TypedInput
          label='required owners'
          hint='number of required owners to accept a transaction'
          value={ wallet.required }
          error={ errors.required }
          onChange={ this.onRequiredChange }
          param={ parseAbiType('uint') }
        />

        <TypedInput
          label='wallet day limit'
          hint='number of days to wait for other owners confirmation'
          value={ wallet.daylimit }
          error={ errors.daylimit }
          onChange={ this.onDaylimitChange }
          param={ parseAbiType('uint') }
        />
      </Form>
    );
  }

  onAccoutChange = (_, account) => {
    this.props.onChange({ account });
  }

  onNameChange = (_, name) => {
    this.props.onChange({ name });
  }

  onDescriptionChange = (_, description) => {
    this.props.onChange({ description });
  }

  onOwnersChange = (owners) => {
    this.props.onChange({ owners });
  }

  onRequiredChange = (required) => {
    this.props.onChange({ required });
  }

  onDaylimitChange = (daylimit) => {
    this.props.onChange({ daylimit });
  }
}
