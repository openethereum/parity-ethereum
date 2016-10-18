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

import { RaisedButton } from 'material-ui';
import ActionAddShoppingCart from 'material-ui/svg-icons/action/add-shopping-cart';
// import AvReplay from 'material-ui/svg-icons/av/replay';
import ContentSend from 'material-ui/svg-icons/content/send';

import styles from './actions.css';

export default class Actions extends Component {
  static propTypes = {
    onAction: PropTypes.func.isRequired,
    gavBalance: PropTypes.object.isRequired
  }

  render () {
    const { gavBalance } = this.props;

    return (
      <div className={ styles.actions }>
        <RaisedButton
          className={ styles.button }
          icon={ <ActionAddShoppingCart /> }
          label='buy coins'
          primary
          onTouchTap={ this.onBuyIn } />
        <RaisedButton
          disabled={ !gavBalance || gavBalance.eq(0) }
          className={ styles.button }
          icon={ <ContentSend /> }
          label='send coins'
          primary
          onTouchTap={ this.onTransfer } />
      </div>
    );

    // <RaisedButton
    //   className={ styles.button }
    //   icon={ <AvReplay /> }
    //   label='claim refund'
    //   primary
    //   onTouchTap={ this.onRefund } />
  }

  onBuyIn = () => {
    this.props.onAction('BuyIn');
  }

  onTransfer = () => {
    const { gavBalance } = this.props;

    if (gavBalance && gavBalance.gt(0)) {
      this.props.onAction('Transfer');
    }
  }

  onRefund = () => {
    this.props.onAction('Refund');
  }
}
