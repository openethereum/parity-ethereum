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

export default class CodeStep extends Component {
  static propTypes = {
    abi: PropTypes.string,
    abiError: PropTypes.string,
    code: PropTypes.string,
    codeError: PropTypes.string,
    onAbiChange: PropTypes.func.isRequired,
    onCodeChange: PropTypes.func.isRequired
  }

  render () {
    const { abi, abiError, code, codeError, onAbiChange, onCodeChange } = this.props;

    return (
      <Form>
        <Input
          label='abi'
          hint='the abi of the contract to deploy'
          error={ abiError }
          value={ abi }
          onSubmit={ onAbiChange } />
        <Input
          label='code'
          hint='the compiled code of the contract to deploy'
          error={ codeError }
          value={ code }
          onSubmit={ onCodeChange } />
      </Form>
    );
  }
}
