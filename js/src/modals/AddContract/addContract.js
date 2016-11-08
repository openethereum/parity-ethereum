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
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Button, Modal, Form, Input, InputAddress } from '../../ui';
import { ERRORS, validateAbi, validateAddress, validateName } from '../../util/validation';

export default class AddContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    contracts: PropTypes.object.isRequired,
    onClose: PropTypes.func
  };

  state = {
    abi: '',
    abiError: ERRORS.invalidAbi,
    abiParsed: null,
    address: '',
    addressError: ERRORS.invalidAddress,
    name: '',
    nameError: ERRORS.invalidName,
    description: ''
  };

  render () {
    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        title='watch contract'>
        { this.renderFields() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { addressError, nameError } = this.state;
    const hasError = !!(addressError || nameError);

    return ([
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose } />,
      <Button
        icon={ <ContentAdd /> }
        label='Add Contract'
        disabled={ hasError }
        onClick={ this.onAdd } />
    ]);
  }

  renderFields () {
    const { abi, abiError, address, addressError, description, name, nameError } = this.state;

    return (
      <Form>
        <InputAddress
          label='network address'
          hint='the network address for the contract'
          error={ addressError }
          value={ address }
          onSubmit={ this.onEditAddress } />
        <Input
          label='contract name'
          hint='a descriptive name for the contract'
          error={ nameError }
          value={ name }
          onSubmit={ this.onEditName } />
        <Input
          multiLine
          rows={ 1 }
          label='(optional) contract description'
          hint='an expanded description for the entry'
          value={ description }
          onSubmit={ this.onEditDescription } />
        <Input
          label='contract abi'
          hint='the abi for the contract'
          error={ abiError }
          value={ abi }
          onSubmit={ this.onEditAbi } />
      </Form>
    );
  }

  onEditAbi = (abi) => {
    const { api } = this.context;

    this.setState(validateAbi(abi, api));
  }

  onEditAddress = (_address) => {
    const { contracts } = this.props;
    let { address, addressError } = validateAddress(_address);

    if (!addressError) {
      const contract = contracts[address];

      if (contract && !contract.meta.deleted) {
        addressError = ERRORS.duplicateAddress;
      }
    }

    this.setState({
      address,
      addressError
    });
  }

  onEditDescription = (description) => {
    this.setState({ description });
  }

  onEditName = (name) => {
    this.setState(validateName(name));
  }

  onAdd = () => {
    const { api } = this.context;
    const { abiParsed, address, name, description } = this.state;

    Promise.all([
      api.parity.setAccountName(address, name),
      api.parity.setAccountMeta(address, {
        contract: true,
        deleted: false,
        timestamp: Date.now(),
        abi: abiParsed,
        description
      })
    ]).catch((error) => {
      console.error('onAdd', error);
    });

    this.props.onClose();
  }

  onClose = () => {
    this.props.onClose();
  }
}
