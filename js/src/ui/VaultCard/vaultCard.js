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
import { Link } from 'react-router';

import Button from '~/ui/Button';
import Container from '~/ui/Container';
import IdentityIcon from '~/ui/IdentityIcon';
import { LockedIcon, UnlockedIcon } from '~/ui/Icons';

import Layout from './Layout';
import styles from './vaultCard.css';

export default class VaultCard extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    buttons: PropTypes.array,
    children: PropTypes.node,
    hideAccounts: PropTypes.bool,
    hideButtons: PropTypes.bool,
    vault: PropTypes.object.isRequired
  };

  static Layout = Layout;

  render () {
    const { children, vault } = this.props;
    const { isOpen } = vault;

    return (
      <Container
        className={ styles.container }
        hover={
          isOpen
            ? this.renderAccounts()
            : null
        }
      >
        { this.renderButtons() }
        <Layout vault={ vault }>
          { children }
        </Layout>
      </Container>
    );
  }

  renderAccounts () {
    const { accounts, hideAccounts } = this.props;

    if (hideAccounts) {
      return null;
    }

    if (!accounts || !accounts.length) {
      return (
        <div className={ styles.empty }>
          <FormattedMessage
            id='vaults.accounts.empty'
            defaultMessage='There are no accounts in this vault'
          />
        </div>
      );
    }

    return (
      <div className={ styles.accounts }>
        {
          accounts.map((address) => {
            return (
              <Link
                key={ address }
                to={ `/accounts/${address}` }
              >
                <IdentityIcon
                  address={ address }
                  center
                  className={ styles.account }
                />
              </Link>
            );
          })
        }
      </div>
    );
  }

  renderButtons () {
    const { buttons, hideButtons, vault } = this.props;
    const { isOpen } = vault;

    if (hideButtons) {
      return null;
    }

    return (
      <div className={ styles.buttons }>
        <Button
          className={ styles.status }
          disabled
          icon={
            isOpen
              ? <UnlockedIcon />
              : <LockedIcon />
          }
          key='status'
        />
        { buttons }
      </div>
    );
  }
}
