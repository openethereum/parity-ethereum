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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { newError } from '@parity/shared/redux/actions';
import { ConfirmDialog, IdentityIcon, IdentityName, Input } from '@parity/ui';

import styles from './deleteAccount.css';

class DeleteAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    router: PropTypes.object
  }

  static propTypes = {
    account: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired,
    newError: PropTypes.func.isRequired
  }

  state = {
    isBusy: false,
    password: ''
  }

  render () {
    const { account } = this.props;
    const { isBusy, password } = this.state;

    return (
      <ConfirmDialog
        busy={ isBusy }
        className={ styles.body }
        onConfirm={ this.onDeleteConfirmed }
        onDeny={ this.closeDeleteDialog }
        open
        title={
          <FormattedMessage
            id='deleteAccount.title'
            defaultMessage='confirm removal'
          />
        }
      >
        <div className={ styles.hero }>
          <FormattedMessage
            id='deleteAccount.question'
            defaultMessage='Are you sure you want to permanently delete the following account?'
          />
        </div>
        <div className={ styles.info }>
          <IdentityIcon
            address={ account.address }
            className={ styles.icon }
          />
          <div className={ styles.nameinfo }>
            <div className={ styles.header }>
              <IdentityName
                address={ account.address }
                unknown
              />
            </div>
            <div className={ styles.address }>
              { account.address }
            </div>
          </div>
        </div>
        <div className={ styles.description }>
          { account.meta.description }
        </div>
        <div className={ styles.password }>
          <Input
            autoFocus
            hint={
              <FormattedMessage
                id='deleteAccount.password.hint'
                defaultMessage='provide the account password to confirm the account deletion'
              />
            }
            label={
              <FormattedMessage
                id='deleteAccount.password.label'
                defaultMessage='account password'
              />
            }
            onChange={ this.onChangePassword }
            onDefaultAction={ this.onDeleteConfirmed }
            type='password'
            value={ password }
          />
        </div>
      </ConfirmDialog>
    );
  }

  onChangePassword = (event, password) => {
    this.setState({ password });
  }

  onDeleteConfirmed = () => {
    const { api, router } = this.context;
    const { account, newError } = this.props;
    const { password } = this.state;

    this.setState({ isBusy: true });

    return api.parity
      .killAccount(account.address, password)
      .then((result) => {
        this.setState({ isBusy: true });

        if (result === true) {
          router.push('/accounts');
          this.closeDeleteDialog();
        } else {
          newError(new Error('Deletion failed.'));
        }
      })
      .catch((error) => {
        this.setState({ isBusy: false });
        console.error('onDeleteConfirmed', error);
        newError(new Error(`Deletion failed: ${error.message}`));
      });
  }

  closeDeleteDialog = () => {
    this.props.onClose();
  }
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ newError }, dispatch);
}

export default connect(
  null,
  mapDispatchToProps
)(DeleteAccount);
