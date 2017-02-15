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
import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';
import ActionTrackChanges from 'material-ui/svg-icons/action/track-changes';
import ActionSettings from 'material-ui/svg-icons/action/settings';
import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';
import ImageGridOn from 'material-ui/svg-icons/image/grid-on';
import NavigationApps from 'material-ui/svg-icons/navigation/apps';

import imagesEthcoreBlock from '~/../assets/images/parity-logo-white-no-text.svg';

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
    icon: <ActionAccountBalanceWallet />,
    route: '/accounts',
    value: 'account'
  },

  addresses: {
    active: true,
    icon: <CommunicationContacts />,
    route: '/addresses',
    value: 'address'
  },

  apps: {
    active: true,
    icon: <NavigationApps />,
    route: '/apps',
    value: 'app'
  },

  contracts: {
    active: false,
    icon: <ImageGridOn />,
    route: '/contracts',
    value: 'contract'
  },

  status: {
    active: false,
    icon: <ActionTrackChanges />,
    route: '/status',
    value: 'status'
  },

  signer: {
    active: true,
    fixed: true,
    icon: <ActionFingerprint />,
    route: '/signer',
    value: 'signer'
  },

  settings: {
    active: true,
    fixed: true,
    icon: <ActionSettings />,
    route: '/settings',
    value: 'settings'
  }
};

export default defaultViews;
