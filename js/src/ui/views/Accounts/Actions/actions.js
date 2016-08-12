import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import ContentAdd from 'material-ui/svg-icons/content/add';

export default class Actions extends Component {
  static propTypes = {
    onNewAccount: PropTypes.func.isRequired
  }

  render () {
    return (
      <Toolbar>
        <ToolbarGroup>
          <FlatButton
            icon={ <ContentAdd /> }
            label='new account'
            primary
            onTouchTap={ this.props.onNewAccount } />
        </ToolbarGroup>
      </Toolbar>
    );
  }
}
