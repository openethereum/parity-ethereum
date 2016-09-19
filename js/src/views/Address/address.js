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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ActionDelete from 'material-ui/svg-icons/action/delete';

import { newError } from '../../redux/actions';
import { Actionbar, Button, ConfirmDialog, IdentityIcon, Page } from '../../ui';

import Header from '../Account/Header';
import Transactions from '../Account/Transactions';

import styles from './address.css';

class Address extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    router: PropTypes.object.isRequired
  }

  static propTypes = {
    contacts: PropTypes.object,
    balances: PropTypes.object,
    isTest: PropTypes.bool,
    params: PropTypes.object
  }

  state = {
    showDeleteDialog: false
  }

  render () {
    const { contacts, balances, isTest } = this.props;
    const { address } = this.props.params;

    const contact = (contacts || {})[address];
    const balance = (balances || {})[address];

    if (!contact) {
      return null;
    }

    return (
      <div className={ styles.address }>
        { this.renderActionbar(contact) }
        { this.renderDeleteConfirm() }
        <Page>
          <Header
            isTest={ isTest }
            account={ contact }
            balance={ balance } />
          <Transactions
            address={ address } />
        </Page>
      </div>
    );
  }

  renderActionbar (contact) {
    const buttons = [
      <Button
        key='delete'
        icon={ <ActionDelete /> }
        label='delete address'
        onClick={ this.showDeleteDialog } />
    ];

    return (
      <Actionbar
        title='Address Information'
        buttons={ contact.meta.deleted ? [] : buttons } />
    );
  }

  renderDeleteConfirm () {
    const { contacts } = this.props;
    const { showDeleteDialog } = this.state;

    if (!showDeleteDialog) {
      return;
    }

    const { address } = this.props.params;
    const contact = contacts[address];

    return (
      <ConfirmDialog
        className={ styles.delete }
        title='confirm removal'
        visible
        onDeny={ this.closeDeleteDialog }
        onConfirm={ this.onDeleteConfirmed }>
        <div className={ styles.hero }>
          Are you sure you want to remove the following address from your addressbook?
        </div>
        <div className={ styles.info }>
          <IdentityIcon
            className={ styles.icon }
            address={ address } />
          <div className={ styles.nameinfo }>
            <div className={ styles.header }>
              { contact.name || 'Unnamed' }
            </div>
            <div className={ styles.address }>
              { address }
            </div>
          </div>
        </div>
        <div className={ styles.description }>
          { contact.meta.description }
        </div>
      </ConfirmDialog>
    );
  }

  onDeleteConfirmed = () => {
    const { api, router } = this.context;
    const { contacts } = this.props;
    const { address } = this.props.params;
    const contact = (contacts || {})[address];

    this.toggleDeleteDialog();
    contact.meta.deleted = true;

    api.personal
      .setAccountMeta(address, contact.meta)
      .then(() => router.push('/addresses'))
      .catch((error) => {
        console.error('onDeleteConfirmed', error);
        newError(new Error(`Deletion failed: ${error.message}`));
      });
  }

  closeDeleteDialog = () => {
    this.setState({ showDeleteDialog: false });
  }

  showDeleteDialog = () => {
    this.setState({ showDeleteDialog: true });
  }
}

function mapStateToProps (state) {
  const { contacts } = state.personal;
  const { balances } = state.balances;
  const { isTest } = state.nodeStatus;

  return {
    isTest,
    contacts,
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ newError }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Address);
