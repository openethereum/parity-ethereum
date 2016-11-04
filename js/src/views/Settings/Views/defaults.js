// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

const defaultViews = {
  accounts: {
    active: true,
    fixed: true,
    icon: <ActionAccountBalanceWallet />,
    label: 'Accounts',
    route: '/accounts',
    value: 'account',
    description: 'A list of all the accounts associated to and imported into this Parity instance. Send transactions, receive incoming values, manage your balances and fund your accounts.'
  },

  addresses: {
    active: true,
    icon: <CommunicationContacts />,
    label: 'Addressbook',
    route: '/addresses',
    value: 'address',
    description: 'A list of all contacts and address book entries that is managed by this Parity instance. Watch accounts and have the details available at the click of a button when transacting.'
  },

  apps: {
    active: true,
    icon: <NavigationApps />,
    label: 'Applications',
    route: '/apps',
    value: 'app',
    description: 'Distributed applications that interact with the underlying network. Add applications, manage you application portfolio and interact with application from around the newtork.'
  },

  contracts: {
    active: false,
    icon: <ImageGridOn />,
    label: 'Contracts',
    route: '/contracts',
    value: 'contract',
    description: 'Watch and interact with specific contracts that have been deployed on the network. This is a more technically-focussed environment, specifically for advanced users that understand the inner working of certain contracts.'
  },

  status: {
    active: false,
    icon: <ActionTrackChanges />,
    label: 'Status',
    route: '/status',
    value: 'status',
    description: 'See how the Parity node is performing in terms of connections to the network, logs from the actual running instance and details of mining (if enabled and configured).'
  },

  signer: {
    active: true,
    fixed: true,
    icon: <ActionFingerprint />,
    label: 'Signer',
    route: '/signer',
    value: 'signer',
    description: 'The security focussed area of the application where you can approve any outgoing transactions made from the application as well as those placed into the queue by distributed applications.'
  },

  settings: {
    active: true,
    fixed: true,
    icon: <ActionSettings />,
    label: 'Settings',
    route: '/settings',
    value: 'settings',
    description: 'This view. Allows you to customize the application in term of options, operation and look and feel.'
  }
};

export default defaultViews;
