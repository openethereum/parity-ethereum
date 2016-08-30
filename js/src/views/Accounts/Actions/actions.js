import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';
import ContentAdd from 'material-ui/svg-icons/content/add';

import Tooltip from '../../../ui/Tooltip';

import styles from '../style.css';

export default class Actions extends Component {
  static propTypes = {
    onAddressBook: PropTypes.func.isRequired,
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
          <FlatButton
            icon={ <CommunicationContacts /> }
            label='address book'
            primary
            onTouchTap={ this.props.onAddressBook } />
        </ToolbarGroup>
        <Tooltip
          left='5%' top='85%'
          text='actions relating to the current view are available on the toolbar for quick access, be it for performing actions or creating a new item' />
      </Toolbar>
    );
  }
}
