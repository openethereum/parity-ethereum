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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { AccountCard, Portal, SectionList } from '~/ui';
import { CheckIcon, StarIcon, StarOutlineIcon } from '~/ui/Icons';

import styles from './dappPermissions.css';

@observer
class DappPermissions extends Component {
  static propTypes = {
    balances: PropTypes.object,
    permissionStore: PropTypes.object.isRequired
  };

  render () {
    const { permissionStore } = this.props;

    if (!permissionStore.modalOpen) {
      return null;
    }

    return (
      <Portal
        buttons={
          <div className={ styles.legend }>
            <FormattedMessage
              id='dapps.permissions.description'
              defaultMessage='{activeIcon} account is available to application, {defaultIcon} account is the default account'
              values={ {
                activeIcon: <CheckIcon />,
                defaultIcon: <StarIcon />
              } }
            />
          </div>
        }
        onClose={ permissionStore.closeModal }
        open
        title={
          <FormattedMessage
            id='dapps.permissions.label'
            defaultMessage='visible dapp accounts'
          />
        }
      >
        <div className={ styles.container }>
          <SectionList
            items={ permissionStore.accounts }
            noStretch
            renderItem={ this.renderAccount }
          />
        </div>
      </Portal>
    );
  }

  renderAccount = (account) => {
    const { balances, permissionStore } = this.props;
    const balance = balances[account.address];

    const onMakeDefault = () => {
      permissionStore.setDefaultAccount(account.address);
    };

    const onSelect = () => {
      permissionStore.selectAccount(account.address);
    };

    let className;

    if (account.checked) {
      className = account.default
        ? `${styles.selected} ${styles.default}`
        : styles.selected;
    } else {
      className = styles.unselected;
    }

    return (
      <div className={ styles.item }>
        <AccountCard
          account={ account }
          balance={ balance }
          className={ className }
          onClick={ onSelect }
        />
        <div className={ styles.overlay }>
          {
            account.checked && account.default
              ? <StarIcon />
              : <StarOutlineIcon className={ styles.iconDisabled } onClick={ onMakeDefault } />
          }
          {
            account.checked
              ? <CheckIcon onClick={ onSelect } />
              : <CheckIcon className={ styles.iconDisabled } onClick={ onSelect } />
          }
        </div>
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { balances } = state.balances;

  return {
    balances
  };
}

export default connect(
  mapStateToProps,
  null
)(DappPermissions);
