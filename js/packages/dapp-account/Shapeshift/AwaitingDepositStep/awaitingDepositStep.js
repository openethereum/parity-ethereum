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

import { observer } from 'mobx-react';
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { CopyToClipboard, QrCode } from '@parity/ui';

import Value from '../Value';
import styles from '../shapeshift.css';

@observer
export default class AwaitingDepositStep extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { coinSymbol, depositAddress, price } = this.props.store;
    const typeSymbol = (
      <div className={ styles.symbol }>
        { coinSymbol }
      </div>
    );

    if (!depositAddress) {
      return (
        <div className={ styles.center }>
          <div className={ styles.busy }>
            <FormattedMessage
              id='shapeshift.awaitingDepositStep.awaitingConfirmation'
              defaultMessage='Awaiting confirmation of the deposit address for your {typeSymbol} funds exchange'
              values={ { typeSymbol } }
            />
          </div>
        </div>
      );
    }

    return (
      <div className={ styles.center }>
        <div className={ styles.info }>
          <FormattedMessage
            id='shapeshift.awaitingDepositStep.awaitingDeposit'
            defaultMessage='{shapeshiftLink} is awaiting a {typeSymbol} deposit. Send the funds from your {typeSymbol} network client to -'
            values={ {
              shapeshiftLink: <a href='https://shapeshift.io' target='_blank'>ShapeShift.io</a>,
              typeSymbol
            } }
          />
        </div>
        { this.renderAddress(depositAddress, coinSymbol) }
        <div className={ styles.price }>
          <div>
            <FormattedMessage
              id='shapeshift.awaitingDepositStep.minimumMaximum'
              defaultMessage='{minimum} minimum, {maximum} maximum'
              values={ {
                maximum: <Value amount={ price.limit } symbol={ coinSymbol } />,
                minimum: <Value amount={ price.minimum } symbol={ coinSymbol } />
              } }
            />
          </div>
        </div>
      </div>
    );
  }

  renderAddress (depositAddress, coinSymbol) {
    const qrcode = (
      <QrCode
        className={ styles.qrcode }
        value={ depositAddress }
      />
    );
    let protocolLink = null;

    // TODO: Expand for other coins where protocols are available
    switch (coinSymbol) {
      case 'BTC':
        protocolLink = `bitcoin:${depositAddress}`;
        break;
    }

    return (
      <div className={ styles.addressInfo }>
        {
          protocolLink
            ? (
              <a
                href={ protocolLink }
                target='_blank'
              >
                { qrcode }
              </a>
            )
            : qrcode
        }
        <div className={ styles.address }>
          <CopyToClipboard data={ depositAddress } />
          <span>{ depositAddress }</span>
        </div>
      </div>
    );
  }
}
