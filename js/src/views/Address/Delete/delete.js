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
import { ConfirmDialog, IdentityIcon, IdentityName } from '@parity/ui';

import styles from '../address.css';

class Delete extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    router: PropTypes.object
  };

  static propTypes = {
    route: PropTypes.string.isRequired,

    address: PropTypes.string,
    account: PropTypes.object,
    confirmMessage: PropTypes.node,
    visible: PropTypes.bool,
    onClose: PropTypes.func,
    newError: PropTypes.func
  };

  state = {
    isBusy: false
  };

  render () {
    const { account, confirmMessage, visible } = this.props;
    const { isBusy } = this.state;

    if (!visible) {
      return null;
    }

    return (
      <ConfirmDialog
        busy={ isBusy }
        className={ styles.delete }
        title={
          <FormattedMessage
            id='address.delete.title'
            defaultMessage='confirm removal'
          />
        }
        visible
        onDeny={ this.closeDeleteDialog }
        onConfirm={ this.onDeleteConfirmed }
      >
        <div className={ styles.hero }>
          {
            confirmMessage || (
              <FormattedMessage
                id='address.delete.confirmInfo'
                defaultMessage='Are you sure you want to remove the following address from your addressbook?'
              />
            )
          }
        </div>
        <div className={ styles.info }>
          <IdentityIcon
            className={ styles.icon }
            address={ account.address }
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
      </ConfirmDialog>
    );
  }

  onDeleteConfirmed = () => {
    const { api, router } = this.context;
    const { account, route, newError } = this.props;

    this.setState({ isBusy: true });

    api.parity
      .removeAddress(account.address)
      .then(() => {
        router.push(route);
        this.closeDeleteDialog();
      })
      .catch((error) => {
        console.error('onDeleteConfirmed', error);
        newError(new Error(`Deletion failed: ${error.message}`));
        this.closeDeleteDialog();
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
)(Delete);
