import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import ContentAdd from 'material-ui/svg-icons/content/add';

export default class Actions extends Component {
  static propTypes = {
    onAction: PropTypes.func.isRequired
  }

  render () {
    return (
      <Toolbar className='actions'>
        <ToolbarGroup>
          <FlatButton
            icon={ <ContentAdd /> }
            label='buy coins'
            primary
            onTouchTap={ this.onBuyIn } />
          <FlatButton
            icon={ <ContentAdd /> }
            label='transfer coins'
            primary disabled
            onTouchTap={ this.onTransfer } />
          <FlatButton
            icon={ <ContentAdd /> }
            label='claim refund'
            primary disabled
            onTouchTap={ this.onRefund } />
        </ToolbarGroup>
      </Toolbar>
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
