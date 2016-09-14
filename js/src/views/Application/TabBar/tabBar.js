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

import React, { Component, PropTypes } from 'react';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { Tabs, Tab } from 'material-ui/Tabs';
import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import ActionTrackChanges from 'material-ui/svg-icons/action/track-changes';
import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';
import NavigationApps from 'material-ui/svg-icons/navigation/apps';

import { Badge, SignerIcon, Tooltip } from '../../../ui';

import styles from './tabBar.css';
import imagesEthcoreBlock from '../../../images/ethcore-block.png';

const TABMAP = {
  accounts: 'account',
  addresses: 'address',
  apps: 'app',
  contracts: 'contract'
};

export default class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  static propTypes = {
    pending: PropTypes.array,
    isTest: PropTypes.bool,
    netChain: PropTypes.string
  }

  render () {
    const windowHash = (window.location.hash || '')
      .split('?')[0].split('/')[1];
    const hash = TABMAP[windowHash] || windowHash;

    return (
      <Toolbar
        className={ styles.toolbar }>
        <ToolbarGroup>
          <div className={ styles.logo }>
            <img src={ imagesEthcoreBlock } />
            <div>Parity</div>
          </div>
        </ToolbarGroup>
        <Tabs
          className={ styles.tabs }
          value={ hash }>
          <Tab
            className={ hash === 'account' ? styles.tabactive : '' }
            data-route='/accounts'
            value='account'
            icon={ <ActionAccountBalanceWallet /> }
            label={ this.renderLabel('accounts') }
            onActive={ this.onActivate }>
            <Tooltip
              className={ styles.tabbarTooltip }
              text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />
          </Tab>
          <Tab
            className={ hash === 'address' ? styles.tabactive : '' }
            data-route='/addresses'
            value='address'
            icon={ <CommunicationContacts /> }
            label={ this.renderLabel('address book') }
            onActive={ this.onActivate } />
          <Tab
            className={ hash === 'app' ? styles.tabactive : '' }
            data-route='/apps'
            value='app'
            icon={ <NavigationApps /> }
            label={ this.renderLabel('apps') }
            onActive={ this.onActivate } />
          <Tab
            className={ hash === 'status' ? styles.tabactive : '' }
            data-route='/status'
            value='status'
            icon={ <ActionTrackChanges /> }
            label={ this.renderStatusLabel() }
            onActive={ this.onActivate } />
          <Tab
            className={ hash === 'signer' ? styles.tabactive : '' }
            data-route='/signer'
            value='signer'
            icon={ <SignerIcon className={ styles.signerIcon } /> }
            label={ this.renderSignerLabel() }
            onActive={ this.onActivate } />
        </Tabs>
      </Toolbar>
    );
  }

  renderLabel (name, bubble) {
    return (
      <div className={ styles.label }>
        { name }
        { bubble }
      </div>
    );
  }

  renderSignerLabel () {
    const { pending } = this.props;
    let bubble = null;

    if (pending && pending.length) {
      bubble = (
        <Badge
          color='red'
          className={ styles.labelBubble }
          value={ pending.length } />
      );
    }

    return this.renderLabel('signer', bubble);
  }

  renderStatusLabel () {
    const { isTest, netChain } = this.props;
    const bubble = (
      <Badge
        color={ isTest ? 'red' : 'default' }
        className={ styles.labelBubble }
        value={ isTest ? 'TEST' : netChain } />
      );

    return this.renderLabel('status', bubble);
  }

  onActivate = (tab) => {
    const { router } = this.context;

    router.push(tab.props['data-route']);
  }
}
