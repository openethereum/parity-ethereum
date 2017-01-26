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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { ConfirmDialog, IdentityIcon, IdentityName, Input } from '~/ui';
import { newError } from '~/redux/actions';

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
    password: ''
  }

  render () {
    const { account } = this.props;
    const { password } = this.state;

    return (
      <ConfirmDialog
        className={ styles.body }
        title='confirm removal'
        visible
        onDeny={ this.closeDeleteDialog }
        onConfirm={ this.onDeleteConfirmed }
      >
        <div className={ styles.hero }>
          Are you sure you want to permanently delete the following account?
        </div>
        <div className={ styles.info }>
          <IdentityIcon
            className={ styles.icon }
            address={ account.address }
          />
          <div className={ styles.nameinfo }>
            <div className={ styles.header }>
              <IdentityName address={ account.address } unknown />
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
            label='account password'
            hint='provide the account password to confirm the account deletion'
            type='password'
            value={ password }
            onChange={ this.onChangePassword }
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

    api.parity
      .killAccount(account.address, password)
      .then((result) => {
        if (result === true) {
          router.push('/accounts');
          this.closeDeleteDialog();
        } else {
          newError(new Error('Deletion failed.'));
        }
      })
      .catch((error) => {
        console.error('onDeleteConfirmed', error);
        newError(new Error(`Deletion failed: ${error.message}`));
      });
  }

  closeDeleteDialog = () => {
    this.props.onClose();
  }
}

function mapStateToProps (state) {
  return {};
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ newError }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(DeleteAccount);
