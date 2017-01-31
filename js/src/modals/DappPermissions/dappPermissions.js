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

import { AccountCard, ContainerTitle, Portal, SectionList } from '~/ui';
import { CheckIcon, StarIcon, StarOutlineIcon } from '~/ui/Icons';

import styles from './dappPermissions.css';

@observer
export default class DappPermissions extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  };

  render () {
    const { store } = this.props;

    if (!store.modalOpen) {
      return null;
    }

    return (
      <Portal
        className={ styles.modal }
        onClose={ store.closeModal }
        open
      >
        <ContainerTitle
          title={
            <FormattedMessage
              id='dapps.permissions.label'
              defaultMessage='visible dapp accounts'
            />
          }
        />
        <div className={ styles.container }>
          <SectionList
            items={ store.accounts }
            noStretch
            renderItem={ this.renderAccount }
          />
        </div>
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
      </Portal>
    );
  }

  renderAccount = (account) => {
    const { store } = this.props;

    const onMakeDefault = () => {
      store.setDefaultAccount(account.address);
    };

    const onSelect = () => {
      store.selectAccount(account.address);
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
