import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import ContentAdd from 'material-ui/svg-icons/content/add';

import Tooltip from '../../../ui/Tooltip';

import styles from '../style.css';

export default class Actions extends Component {
  static propTypes = {
    onNewAccount: PropTypes.func.isRequired
  }

  render () {
    return (
      <Toolbar
        className={ styles.toolbar }>
        <ToolbarGroup>
          <FlatButton
            icon={ <ContentAdd /> }
            label='new account'
            primary
            onTouchTap={ this.props.onNewAccount } />
        </ToolbarGroup>
        <Tooltip
          text='actions relating to the current view are available on the toolbar for quick access, be it for performing actions or creating a new item' />
      </Toolbar>
    );
  }
}
