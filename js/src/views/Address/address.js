// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { EditMeta, AddAddress } from '~/modals';
import { Actionbar, Button, Page } from '~/ui';
import { AddIcon, DeleteIcon, EditIcon } from '~/ui/Icons';

import Header from '../Account/Header';
import Transactions from '../Account/Transactions';
import Delete from './Delete';
import { setVisibleAccounts } from '~/redux/providers/personalActions';

class Address extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    router: PropTypes.object.isRequired
  };

  static propTypes = {
    setVisibleAccounts: PropTypes.func.isRequired,

    contacts: PropTypes.object,
    balances: PropTypes.object,
    params: PropTypes.object
  };

  state = {
    showDeleteDialog: false,
    showEditDialog: false,
    showAdd: false
  };

  componentDidMount () {
    this.setVisibleAccounts();
  }

  componentWillReceiveProps (nextProps) {
    const prevAddress = this.props.params.address;
    const nextAddress = nextProps.params.address;

    if (prevAddress !== nextAddress) {
      this.setVisibleAccounts(nextProps);
    }
  }

  componentWillUnmount () {
    this.props.setVisibleAccounts([]);
  }

  setVisibleAccounts (props = this.props) {
    const { params, setVisibleAccounts } = props;
    const addresses = [ params.address ];

    setVisibleAccounts(addresses);
  }

  render () {
    const { contacts, balances } = this.props;
    const { address } = this.props.params;

    if (Object.keys(contacts).length === 0) {
      return null;
    }

    const contact = (contacts || {})[address];
    const balance = (balances || {})[address];

    return (
      <div>
        { this.renderAddAddress(contact, address) }
        { this.renderEditDialog(contact) }
        { this.renderActionbar(contact) }
        { this.renderDelete(contact) }
        <Page padded>
          <Header
            account={ contact || { address, meta: {} } }
            balance={ balance }
            hideName={ !contact }
          />
          <Transactions
            address={ address }
          />
        </Page>
      </div>
    );
  }

  renderAddAddress (contact, address) {
    if (contact) {
      return null;
    }

    const { contacts } = this.props;
    const { showAdd } = this.state;

    if (!showAdd) {
      return null;
    }

    return (
      <AddAddress
        contacts={ contacts }
        onClose={ this.onCloseAdd }
        address={ address }
      />
    );
  }

  renderDelete (contact) {
    if (!contact) {
      return null;
    }

    const { showDeleteDialog } = this.state;

    return (
      <Delete
        account={ contact }
        visible={ showDeleteDialog }
        route='/addresses'
        onClose={ this.closeDeleteDialog }
      />
    );
  }

  renderActionbar (contact) {
    const buttons = [
      <Button
        key='editmeta'
        icon={ <EditIcon /> }
        label={
          <FormattedMessage
            id='address.buttons.edit'
            defaultMessage='edit'
          />
        }
        onClick={ this.onEditClick }
      />,
      <Button
        key='delete'
        icon={ <DeleteIcon /> }
        label={
          <FormattedMessage
            id='address.buttons.forget'
            defaultMessage='forget'
          />
        }
        onClick={ this.showDeleteDialog }
      />
    ];

    const addToBook = (
      <Button
        key='newAddress'
        icon={ <AddIcon /> }
        label={
          <FormattedMessage
            id='address.buttons.save'
            defaultMessage='save'
          />
        }
        onClick={ this.onOpenAdd }
      />
    );

    return (
      <Actionbar
        title={
          <FormattedMessage
            id='address.title'
            defaultMessage='Address Information'
          />
        }
        buttons={
          !contact
            ? [ addToBook ]
            : buttons
        }
      />
    );
  }

  renderEditDialog (contact) {
    const { showEditDialog } = this.state;

    if (!contact || !showEditDialog) {
      return null;
    }

    return (
      <EditMeta
        account={ contact }
        onClose={ this.onEditClick }
      />
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

  onOpenAdd = () => {
    this.setState({
      showAdd: true
    });
  }

  onCloseAdd = () => {
    this.setState({ showAdd: false });
  }
}

function mapStateToProps (state) {
  const { contacts } = state.personal;
  const { balances } = state.balances;

  return {
    contacts,
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    setVisibleAccounts
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Address);
