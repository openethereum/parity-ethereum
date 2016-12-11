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

import { GasPriceEditor, Form, Input } from '~/ui';

export default class Extras extends Component {
  static propTypes = {
    isEth: PropTypes.bool,
    data: PropTypes.string,
    dataError: PropTypes.string,
    total: PropTypes.string,
    totalError: PropTypes.string,
    onChange: PropTypes.func.isRequired,
    gasStore: PropTypes.object.isRequired
  }

  render () {
    const { gasStore, onChange } = this.props;

    return (
      <Form>
        { this.renderData() }
        <GasPriceEditor
          store={ gasStore }
          onChange={ onChange } />
      </Form>
    );
  }

  renderData () {
    const { isEth, data, dataError } = this.props;

    if (!isEth) {
      return null;
    }

    return (
      <div>
        <Input
          hint='the data to pass through with the transaction'
          label='transaction data'
          value={ data }
          error={ dataError }
          onChange={ this.onEditData } />
      </div>
    );
  }

  onEditData = (event) => {
    this.props.onChange('data', event.target.value);
  }
}
