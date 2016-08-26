import React, { Component, PropTypes } from 'react';

import { RaisedButton } from 'material-ui';
import ActionAddShoppingCart from 'material-ui/svg-icons/action/add-shopping-cart';
import AvReplay from 'material-ui/svg-icons/av/replay';
import ContentSend from 'material-ui/svg-icons/content/send';

export default class Actions extends Component {
  static propTypes = {
    onAction: PropTypes.func.isRequired
  }

  render () {
    return (
      <div className='actions'>
        <RaisedButton
          className='button'
          icon={ <ActionAddShoppingCart /> }
          label='buy coins'
          primary
          onTouchTap={ this.onBuyIn } />
        <RaisedButton
          className='button'
          icon={ <ContentSend /> }
          label='send coins'
          primary disabled
          onTouchTap={ this.onTransfer } />
        <RaisedButton
          className='button'
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
