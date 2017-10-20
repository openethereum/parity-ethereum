// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button, Form, Input, InputAddress, ModalBox, Portal } from '~/ui';
import { AddIcon, AddressesIcon, CancelIcon } from '~/ui/Icons';

import Store from './store';

@observer
export default class AddAddress extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string,
    contacts: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired
  };

  store = new Store(this.context.api, this.props.contacts);

  componentWillMount () {
    if (this.props.address) {
      this.onEditAddress(null, this.props.address);
    }
  }

  render () {
    return (
      <Portal
        buttons={ this.renderDialogActions() }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='addAddress.label'
            defaultMessage='add saved address'
          />
        }
        visible
      >
        { this.renderFields() }
      </Portal>
    );
  }

  renderDialogActions () {
    const { hasError } = this.store;

    return [
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='addAddress.button.close'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.onClose }
      />,
      <Button
        disabled={ hasError }
        icon={ <AddIcon /> }
        key='save'
        label={
          <FormattedMessage
            id='addAddress.button.add'
            defaultMessage='Save Address'
          />
        }
        onClick={ this.onAdd }
      />
    ];
  }

  renderFields () {
    const { address, addressError, description, name, nameError } = this.store;

    return (
      <ModalBox
        icon={ <AddressesIcon /> }
        summary={
          <FormattedMessage
            id='addAddress.header'
            defaultMessage='To add a new entry to your addressbook, you need the network address of the account and can supply an optional description. Once added it will reflect in your address book.'
          />
        }
      >
        <Form>
          <InputAddress
            allowCopy={ false }
            autoFocus
            disabled={ !!this.props.address }
            error={ addressError }
            hint={
              <FormattedMessage
                id='addAddress.input.address.hint'
                defaultMessage='the network address for the entry'
              />
            }
            label={
              <FormattedMessage
                id='addAddress.input.address.label'
                defaultMessage='network address'
              />
            }
            onChange={ this.onEditAddress }
            value={ address }
          />
          <Input
            error={ nameError }
            hint={
              <FormattedMessage
                id='addAddress.input.name.hint'
                defaultMessage='a descriptive name for the entry'
              />
            }
            label={
              <FormattedMessage
                id='addAddress.input.name.label'
                defaultMessage='address name'
              />
            }
            onChange={ this.onEditName }
            value={ name }
          />
          <Input
            hint={
              <FormattedMessage
                id='addAddress.input.description.hint'
                defaultMessage='an expanded description for the entry'
              />
            }
            label={
              <FormattedMessage
                id='addAddress.input.description.label'
                defaultMessage='(optional) address description'
              />
            }
            onChange={ this.onEditDescription }
            value={ description }
          />
        </Form>
      </ModalBox>
    );
  }

  onEditAddress = (event, address) => {
    this.store.setAddress(address);
  }

  onEditDescription = (event, description) => {
    this.store.setDescription(description);
  }

  onEditName = (event, name) => {
    this.store.setName(name);
  }

  onAdd = () => {
    this.store.add();
    this.onClose();
  }

  onClose = () => {
    this.props.onClose();
  }
}
