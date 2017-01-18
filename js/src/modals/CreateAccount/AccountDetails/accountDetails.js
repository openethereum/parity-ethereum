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

import React, { Component, PropTypes } from 'react';

import { Form, Input, InputAddress } from '~/ui';

export default class AccountDetails extends Component {
  static propTypes = {
    address: PropTypes.string,
    name: PropTypes.string,
    phrase: PropTypes.string
  }

  render () {
    const { address, name } = this.props;

    return (
      <Form>
        <Input
          readOnly
          allowCopy
          hint='a descriptive name for the account'
          label='account name'
          value={ name }
        />
        <InputAddress
          disabled
          hint='the network address for the account'
          label='address'
          value={ address }
        />
        { this.renderPhrase() }
      </Form>
    );
  }

  renderPhrase () {
    const { phrase } = this.props;

    if (!phrase) {
      return null;
    }

    return (
      <Input
        readOnly
        allowCopy
        hint='the account recovery phrase'
        label='owner recovery phrase (keep private and secure, it allows full and unlimited access to the account)'
        value={ phrase }
      />
    );
  }
}
