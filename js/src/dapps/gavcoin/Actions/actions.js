import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import ContentAdd from 'material-ui/svg-icons/content/add';

export default class Actions extends Component {
  static propTypes = {
    onBuyin: PropTypes.func,
    onTransfer: PropTypes.func,
    onRefund: PropTypes.func
  }

  render () {
    return (
      <Toolbar className='actions'>
        <ToolbarGroup>
          <FlatButton
            icon={ <ContentAdd /> }
            label='buy coins'
            primary
            onTouchTap={ this.props.onBuyin } />
          <FlatButton
            icon={ <ContentAdd /> }
            label='transfer coins'
            primary
            onTouchTap={ this.props.onTransfer } />
          <FlatButton
            icon={ <ContentAdd /> }
            label='claim refund'
            primary
            onTouchTap={ this.props.onRefund } />
        </ToolbarGroup>
      </Toolbar>
    );
  }
}
