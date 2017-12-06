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

import imagesEthcoreBlock from '~/../assets/images/parity-logo-white-no-text.svg';
import { AccountsIcon, AddressesIcon, ContactsIcon, SettingsIcon } from '~/ui/Icons';

import styles from './views.css';

const defaultViews = {
  home: {
    active: true,
    fixed: true,
    icon: (
      <img
        className={ styles.logoIcon }
        src={ imagesEthcoreBlock }
      />
    ),
    route: '/home',
    value: 'home'
  },

  accounts: {
    active: true,
    fixed: true,
    icon: <AccountsIcon />,
    route: '/accounts',
    value: 'account'
  },

  addresses: {
    active: true,
    icon: <AddressesIcon />,
    route: '/addresses',
    value: 'address'
  },

  contracts: {
    active: false,
    onlyPersonal: true,
    icon: <ContactsIcon />,
    route: '/contracts',
    value: 'contract'
  },

  // signer: {
  //   active: true,
  //   fixed: true,
  //   icon: <FingerprintIcon />,
  //   route: '/signer',
  //   value: 'signer'
  // },

  settings: {
    active: true,
    fixed: true,
    icon: <SettingsIcon />,
    route: '/settings',
    value: 'settings'
  }
};

export default defaultViews;
