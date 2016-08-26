import React, { Component, PropTypes } from 'react';

import { RaisedButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';

export default class Actions extends Component {
  static propTypes = {
    onAction: PropTypes.func.isRequired
  }

  render () {
    return (
      <div className='actions'>
        <RaisedButton
          className='button'
          icon={ <ContentAdd /> }
          label='buy coins'
          primary
          onTouchTap={ this.onBuyIn } />
        <RaisedButton
          className='button'
          icon={ <ContentAdd /> }
          label='transfer coins'
          primary
          onTouchTap={ this.onTransfer } />
        <RaisedButton
          className='button'
          icon={ <ContentAdd /> }
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
