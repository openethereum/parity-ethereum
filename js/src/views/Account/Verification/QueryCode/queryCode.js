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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';
import { Form, Input } from '@parity/ui';

export default class QueryCode extends Component {
  static propTypes = {
    receiver: PropTypes.string.isRequired,
    hint: nodeOrStringProptype(),
    isCodeValid: PropTypes.bool.isRequired,
    setCode: PropTypes.func.isRequired
  }

  static defaultProps = {
    hint: (
      <FormattedMessage
        id='verification.code.hint'
        defaultMessage='Enter the code you received.'
      />
    )
  }

  render () {
    const { receiver, hint, isCodeValid } = this.props;

    return (
      <Form>
        <p>
          <FormattedMessage
            id='verification.code.sent'
            defaultMessage='The verification code has been sent to {receiver}.'
            values={ {
              receiver
            } }
          />
        </p>
        <Input
          autoFocus
          label={
            <FormattedMessage
              id='verification.code.label'
              defaultMessage='verification code'
            />
          }
          hint={ hint }
          error={
            isCodeValid
              ? null
              : (
                <FormattedMessage
                  id='verification.code.error'
                  defaultMessage='invalid code'
                />
              )
          }
          onChange={ this.onChange }
          onSubmit={ this.onSubmit }
        />
      </Form>
    );
  }

  onChange = (_, code) => {
    this.props.setCode(code.trim());
  }

  onSubmit = (code) => {
    this.props.setCode(code.trim());
  }
}
