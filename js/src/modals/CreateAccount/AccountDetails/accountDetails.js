// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { Form, Input, InputAddress } from '~/ui';

@observer
export default class AccountDetails extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { address, name } = this.props.store;

    return (
      <Form>
        <Input
          allowCopy
          hint={
            <FormattedMessage
              id='createAccount.accountDetails.name.hint'
              defaultMessage='a descriptive name for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.accountDetails.name.label'
              defaultMessage='account name'
            />
          }
          readOnly
          value={ name }
        />
        <InputAddress
          disabled
          hint={
            <FormattedMessage
              id='createAccount.accountDetails.address.hint'
              defaultMessage='the network address for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.accountDetails.address.label'
              defaultMessage='address'
            />
          }
          value={ address }
        />
        { this.renderPhrase() }
      </Form>
    );
  }

  renderPhrase () {
    const { phrase } = this.props.store;

    if (!phrase) {
      return null;
    }

    return (
      <Input
        allowCopy
        hint={
          <FormattedMessage
            id='createAccount.accountDetails.phrase.hint'
            defaultMessage='the account recovery phrase'
          />
        }
        label={
          <FormattedMessage
            id='createAccount.accountDetails.phrase.label'
            defaultMessage='owner recovery phrase (keep private and secure, it allows full and unlimited access to the account)'
          />
        }
        readOnly
        value={ phrase }
      />
    );
  }
}
