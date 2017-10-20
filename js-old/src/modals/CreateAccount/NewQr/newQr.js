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

import { Form, Input, InputAddress, QrScan } from '~/ui';

import ChangeVault from '../ChangeVault';

@observer
export default class NewQr extends Component {
  static propTypes = {
    createStore: PropTypes.object.isRequired,
    vaultStore: PropTypes.object.isRequired
  };

  render () {
    const { createStore } = this.props;

    return createStore.qrAddressValid
      ? this.renderInfo()
      : this.renderScanner();
  }

  renderInfo () {
    const { createStore, vaultStore } = this.props;
    const { description, name, nameError, qrAddress } = createStore;

    return (
      <Form>
        <InputAddress
          allowCopy
          readOnly
          hint={
            <FormattedMessage
              id='createAccount.newQr.address.hint'
              defaultMessage='the network address for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.newQr.address.label'
              defaultMessage='address'
            />
          }
          value={ qrAddress }
        />
        <Input
          autoFocus
          error={ nameError }
          hint={
            <FormattedMessage
              id='createAccount.newQr.name.hint'
              defaultMessage='a descriptive name for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.newQr.name.label'
              defaultMessage='account name'
            />
          }
          onChange={ this.onEditAccountName }
          value={ name }
        />
        <Input
          hint={
            <FormattedMessage
              id='createAccount.newQr.description.hint'
              defaultMessage='a description for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.newQr.description.label'
              defaultMessage='account description'
            />
          }
          onChange={ this.onEditAccountDescription }
          value={ description }
        />
        <ChangeVault
          createStore={ createStore }
          vaultStore={ vaultStore }
        />
      </Form>
    );
  }

  renderScanner () {
    return (
      <div>
        <FormattedMessage
          id='createAccount.newQr.summary'
          defaultMessage='Use the built-in machine camera to scan to QR code of the account you wish to attach as an external account. External accounts are signed on the external device.'
        />
        <QrScan onScan={ this.onScan } />
      </div>
    );
  }

  onEditAccountDescription = (event, description) => {
    const { createStore } = this.props;

    createStore.setDescription(description);
  }

  onEditAccountName = (event, name) => {
    const { createStore } = this.props;

    createStore.setName(name);
  }

  onScan = (address) => {
    const { createStore } = this.props;

    console.log('QR scan', address);

    createStore.setQrAddress(address);
  }
}
