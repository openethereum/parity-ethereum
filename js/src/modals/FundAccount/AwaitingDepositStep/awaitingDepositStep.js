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

import Value from '../Value';

import styles from '../fundAccount.css';

export default class AwaitingDepositStep extends Component {
  static propTypes = {
    coinSymbol: PropTypes.string.isRequired,
    depositAddress: PropTypes.string.isRequired,
    price: PropTypes.shape({
      rate: PropTypes.number.isRequired,
      minimum: PropTypes.number.isRequired,
      limit: PropTypes.number.isRequired
    }).isRequired
  }

  render () {
    const { coinSymbol, depositAddress, price } = this.props;
    const label = `send the ${coinSymbol} funds for exchange to (in your ${coinSymbol} network client)`;

    return (
      <div className={ styles.body }>
        <div className={ styles.info }>
          <a href='https://shapeshift.io' target='_blank'>ShapeShift.io</a> is awaiting a <Value symbol={ coinSymbol } /> deposit (<Value amount={ price.minimum } symbol={ coinSymbol } /> minimum, <Value amount={ price.limit } symbol={ coinSymbol } /> maximum).
        </div>
        <Form>
          <Input
            disabled
            label={ label }
            value={ depositAddress } />
        </Form>
      </div>
    );
  }
}
