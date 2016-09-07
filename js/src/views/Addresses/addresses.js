import React, { Component, PropTypes } from 'react';
import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';

import List from '../Accounts/List';
import { AddAddress } from '../../modals';
import { Actionbar } from '../../ui';

import styles from './addresses.css';

export default class Addresses extends Component {
  static contextTypes = {
    api: PropTypes.object,
    contacts: PropTypes.array
  }

  state = {
    showAdd: false
  }

  render () {
    const { contacts } = this.context;

    return (
      <div className={ styles.addresses }>
        { this.renderActionbar() }
        { this.renderAddAddress() }
        <List
          contact
          accounts={ contacts } />
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
      <FlatButton
        key='newAddress'
        icon={ <ContentAdd /> }
        label='new address'
        primary
        onTouchTap={ this.onOpenAdd } />
    ];

    return (
      <Actionbar
        className={ styles.toolbar }
        title='Saved Addresses'
        buttons={ buttons } />
    );
  }

  renderAddAddress () {
    const { showAdd } = this.state;

    if (!showAdd) {
      return null;
    }

    return (
      <AddAddress
        onClose={ this.onCloseAdd } />
    );
  }

  onOpenAdd = () => {
    this.setState({
      showAdd: true
    });
  }

  onCloseAdd = (address, name, description) => {
    const { api } = this.context;

    this.setState({
      showAdd: false
    });

    Promise.all([
      api.personal.setAccountName(address, name),
      api.personal.setAccountMeta(address, { description })
    ]).catch((error) => {
      console.error('updateDetails', error);
    });
  }
}
