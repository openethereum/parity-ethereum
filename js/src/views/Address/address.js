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
import ContentCreate from 'material-ui/svg-icons/content/create';

import { EditMeta } from '../../modals';
import { Actionbar, Button, Page } from '../../ui';

import Header from '../Account/Header';
import Transactions from '../Account/Transactions';
import Delete from './Delete';

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
    showDeleteDialog: false,
    showEditDialog: false
  }

  render () {
    const { contacts, balances, isTest } = this.props;
    const { address } = this.props.params;
    const { showDeleteDialog } = this.state;

    const contact = (contacts || {})[address];
    const balance = (balances || {})[address];

    if (!contact) {
      return null;
    }

    return (
      <div className={ styles.address }>
        { this.renderEditDialog(contact) }
        { this.renderActionbar(contact) }
        <Delete
          account={ contact }
          visible={ showDeleteDialog }
          route='/addresses'
          onClose={ this.closeDeleteDialog } />
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
        key='editmeta'
        icon={ <ContentCreate /> }
        label='edit'
        onClick={ this.onEditClick } />,
      <Button
        key='delete'
        icon={ <ActionDelete /> }
        label='delete address'
        onClick={ this.showDeleteDialog } />
    ];

    return (
      <Actionbar
        title='Address Information'
        buttons={ !contact || contact.meta.deleted ? [] : buttons } />
    );
  }

  renderEditDialog (contact) {
    const { showEditDialog } = this.state;

    if (!showEditDialog) {
      return null;
    }

    return (
      <EditMeta
        account={ contact }
        keys={ ['description'] }
        onClose={ this.onEditClick } />
    );
  }

  onEditClick = () => {
    this.setState({
      showEditDialog: !this.state.showEditDialog
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
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Address);
