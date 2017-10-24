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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { Input } from '~/ui/Form';

export default class ExportInput extends Component {
  static propTypes = {
    account: PropTypes.object.isRequired,
    onClick: PropTypes.func.isRequired,
    onChange: PropTypes.func.isRequired,
    value: PropTypes.string
  };

  render () {
    const { value, onChange } = this.props;

    return (
      <Input
        type='password'
        label={
          <FormattedMessage
            id='account.export.setPassword.label'
            defaultMessage='Password'
          />
        }
        hint={
          <FormattedMessage
            id='account.export.setPassword.hint'
            defaultMessage='Enter password Here'
          />
        }
        value={ value }
        onClick={ this.onClick }
        onChange={ onChange }
      />
    );
  }

  onClick = (event) => {
    const { account, onClick } = this.props;

    event.stopPropagation();

    onClick && onClick(account.address);
  }
}
