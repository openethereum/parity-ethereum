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

import React from 'react';
import PropTypes from 'prop-types';

import Container from '../Container';

import Accounts from './Accounts';
import Buttons from './Buttons';
import Layout from './Layout';

import styles from './vaultCard.css';

export default function VaultCard ({ accounts, buttons, children, hideAccounts, hideButtons, vault }) {
  const { isOpen } = vault;

  return (
    <Container
      className={ styles.container }
      hover={
        isOpen && (
          <Accounts
            accounts={ accounts }
            hideAccounts={ hideAccounts }
          />
        )
      }
    >
      <Buttons
        buttons={ buttons }
        hideButtons={ hideButtons }
        vault={ vault }
      />
      <Layout vault={ vault }>
        { children }
      </Layout>
    </Container>
  );
}

VaultCard.propTypes = {
  accounts: PropTypes.array,
  buttons: PropTypes.array,
  children: PropTypes.node,
  hideAccounts: PropTypes.bool,
  hideButtons: PropTypes.bool,
  vault: PropTypes.object.isRequired
};

VaultCard.Layout = Layout;
