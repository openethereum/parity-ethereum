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

import { Form, Input } from '~/ui';

export default class QueryCode extends Component {
  static propTypes = {
    receiver: PropTypes.string.isRequired,
    hint: PropTypes.string,
    isCodeValid: PropTypes.bool.isRequired,
    setCode: PropTypes.func.isRequired
  }

  static defaultProps = {
    hint: 'Enter the code you received.'
  }

  render () {
    const { receiver, hint, isCodeValid } = this.props;

    return (
      <Form>
        <p>The verification code has been sent to { receiver }.</p>
        <Input
          label={ 'verification code' }
          hint={ hint }
          error={ isCodeValid ? null : 'invalid code' }
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
