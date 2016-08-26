import React, { Component, PropTypes } from 'react';

import { RaisedButton } from 'material-ui';
import ActionAddShoppingCart from 'material-ui/svg-icons/action/add-shopping-cart';
import AvReplay from 'material-ui/svg-icons/av/replay';
import ContentSend from 'material-ui/svg-icons/content/send';

import styles from './style.css';

export default class Actions extends Component {
  static propTypes = {
    onAction: PropTypes.func.isRequired
  }

  render () {
    return (
      <div className={ styles.actions }>
        <RaisedButton
          className={ styles.button }
          icon={ <ActionAddShoppingCart /> }
          label='buy coins'
          primary
          onTouchTap={ this.onBuyIn } />
        <RaisedButton
          className={ styles.button }
          icon={ <ContentSend /> }
          label='send coins'
          primary
          onTouchTap={ this.onTransfer } />
        <RaisedButton
          className={ styles.button }
          icon={ <AvReplay /> }
          label='claim refund'
          primary
          onTouchTap={ this.onRefund } />
      </div>
    );
  }

  onBuyIn = () => {
    this.props.onAction('BuyIn');
  }

  onTransfer = () => {
    this.props.onAction('Transfer');
  }

  onRefund = () => {
    this.props.onAction('Refund');
  }
}
