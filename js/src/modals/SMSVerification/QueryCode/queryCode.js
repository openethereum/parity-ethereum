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

import { Form, Input } from '../../../ui';

const isValidCode = /^[A-Z0-9_-]{7,14}$/i;

export default class QueryCode extends Component {
  static propTypes = {
    data: PropTypes.object.isRequired,
    onData: PropTypes.func.isRequired,
    onDataIsValid: PropTypes.func.isRequired,
    onDataIsInvalid: PropTypes.func.isRequired
  }

  render () {
    const { number, code } = this.props.data;

    return (
      <Form>
        <p>The verification code has been sent to { number }.</p>
        <Input
          label={ 'verification code' }
          hint={ 'Enter the code you received via SMS.' }
          error={ isValidCode.test(code) ? null : 'invalid code' }
          onChange={ this.onChange }
          onSubmit={ this.onSubmit }
        />
      </Form>
    );
  }

  onChange = (_, code) => {
    code = code.trim();
    this.props.onData({ code });

    if (isValidCode.test(code)) {
      this.props.onDataIsValid();
    } else {
      this.props.onDataIsInvalid();
    }
  }
  onSubmit = (code) => {
    this.onChange(null, code);
  }
}
