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

import Value from '../Value';

import styles from '../shapeshift.css';

export default class CompletedStep extends Component {
  static propTypes = {
    depositInfo: PropTypes.shape({
      incomingCoin: PropTypes.number.isRequired,
      incomingType: PropTypes.string.isRequired
    }).isRequired,
    exchangeInfo: PropTypes.shape({
      outgoingCoin: PropTypes.string.isRequired,
      outgoingType: PropTypes.string.isRequired
    }).isRequired
  }

  render () {
    const { depositInfo, exchangeInfo } = this.props;
    const { incomingCoin, incomingType } = depositInfo;
    const { outgoingCoin, outgoingType } = exchangeInfo;

    return (
      <div className={ styles.center }>
        <div className={ styles.info }>
          <a href='https://shapeshift.io' target='_blank'>ShapeShift.io</a> has completed the funds exchange.
        </div>
        <div className={ styles.hero }>
          <Value amount={ incomingCoin } symbol={ incomingType } /> => <Value amount={ outgoingCoin } symbol={ outgoingType } />
        </div>
        <div className={ styles.info }>
          The change in funds will be reflected in your Parity account shortly.
        </div>
      </div>
    );
  }
}
