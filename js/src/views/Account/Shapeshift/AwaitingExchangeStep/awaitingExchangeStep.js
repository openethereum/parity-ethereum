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
import { observer } from 'mobx-react';

import Value from '../Value';
import styles from '../shapeshift.css';

@observer
export default class AwaitingExchangeStep extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { depositInfo } = this.props.store;
    const { incomingCoin, incomingType } = depositInfo;

    return (
      <div className={ styles.center }>
        <div className={ styles.info }>
          <FormattedMessage
            id='shapeshift.awaitingExchangeStep.receivedInfo'
            defaultMessage='{shapeshiftLink} has received a deposit of -'
            values={ {
              shapeshiftLink: <a href='https://shapeshift.io' target='_blank'>ShapeShift.io</a>
            } }
          />
        </div>
        <div className={ styles.hero }>
          <Value amount={ incomingCoin } symbol={ incomingType } />
        </div>
        <div className={ styles.info }>
          <FormattedMessage
            id='shapeshift.awaitingExchangeStep.awaitingCompletion'
            defaultMessage='Awaiting the completion of the funds exchange and transfer of funds to your Parity account.'
          />
        </div>
      </div>
    );
  }
}
