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
import { IconButton, IconMenu, MenuItem } from 'material-ui';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { Tabs, Tab } from 'material-ui/Tabs';
import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import ActionTrackChanges from 'material-ui/svg-icons/action/track-changes';
import ActionSettings from 'material-ui/svg-icons/action/settings';
import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';
import NavigationApps from 'material-ui/svg-icons/navigation/apps';
import RemoveRedEye from 'material-ui/svg-icons/image/remove-red-eye';

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

  state = {
    accountsVisible: true,
    addressesVisible: true,
    appsVisible: true,
    statusVisible: true,
    signerVisible: true,
    activeRoute: '/accounts'
  }

  render () {
    return (
      <Toolbar
        className={ styles.toolbar }>
        { this.renderLogo() }
        { this.renderTabs() }
        { this.renderSettingsMenu() }
      </Toolbar>
    );
  }

  renderLogo () {
    return (
      <ToolbarGroup>
        <div className={ styles.logo }>
          <img src={ imagesEthcoreBlock } />
          <div>Parity</div>
        </div>
      </ToolbarGroup>
    );
  }

  renderSettingsMenu () {
    const items = Object.keys(this.tabs).map((id) => {
      const tab = this.tabs[id];
      const isActive = this.state[`${id}Visible`];
      const icon = (
        <RemoveRedEye className={ isActive ? styles.optionSelected : styles.optionUnselected } />
      );

      return (
        <MenuItem
          className={ isActive ? styles.menuEnabled : styles.menuDisabled }
          leftIcon={ icon }
          key={ id }
          data-id={ id }
          disabled={ tab.fixed }
          primaryText={ tab.label } />
      );
    });

    return (
      <ToolbarGroup>
        <IconMenu
          className={ styles.settings }
          iconButtonElement={ <IconButton><ActionSettings /></IconButton> }
          anchorOrigin={ { horizontal: 'right', vertical: 'bottom' } }
          targetOrigin={ { horizontal: 'right', vertical: 'bottom' } }
          touchTapCloseDelay={ 0 }
          onItemTouchTap={ this.toggleMenu }>
          { items }
        </IconMenu>
      </ToolbarGroup>
    );
  }

  renderTabs () {
    const windowHash = (window.location.hash || '').split('?')[0].split('/')[1];
    const hash = TABMAP[windowHash] || windowHash;

    const items = Object.keys(this.tabs)
      .filter((id) => {
        const tab = this.tabs[id];
        const isFixed = tab.fixed;
        const isVisible = this.state[`${id}Visible`];

        return isFixed || isVisible;
      })
      .map((id) => {
        const tab = this.tabs[id];

        return (
          <Tab
            className={ hash === tab.value ? styles.tabactive : '' }
            data-route={ tab.route }
            value={ tab.value }
            icon={ tab.icon }
            key={ id }
            label={ tab.renderLabel ? tab.renderLabel(tab.label) : this.renderLabel(tab.label) }
            onActive={ this.onActivate }>
            { tab.body }
          </Tab>
        );
      });

    return (
      <Tabs
        className={ styles.tabs }
        value={ hash }>
        { items }
      </Tabs>
    );
  }

  renderLabel = (name, bubble) => {
    return (
      <div className={ styles.label }>
        { name }
        { bubble }
      </div>
    );
  }

  renderSignerLabel = (label) => {
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

    return this.renderLabel(label, bubble);
  }

  renderStatusLabel = (label) => {
    const { isTest, netChain } = this.props;
    const bubble = (
      <Badge
        color={ isTest ? 'red' : 'default' }
        className={ styles.labelBubble }
        value={ isTest ? 'TEST' : netChain } />
      );

    return this.renderLabel(label, bubble);
  }

  onActivate = (tab) => {
    const { router } = this.context;
    const activeRoute = tab.props['data-route'];

    router.push(activeRoute);
    this.setState(activeRoute);
  }

  toggleMenu = (event, menu) => {
    const id = menu.props['data-id'];
    const toggle = `${id}Visible`;
    const isActive = this.state[toggle];

    if (this.tabs[id].fixed) {
      return;
    }

    this.setState({
      [toggle]: !isActive
    });
  }

  tabs = {
    accounts: {
      icon: <ActionAccountBalanceWallet />,
      label: 'Accounts',
      route: '/accounts',
      value: 'account',
      body: <Tooltip className={ styles.tabbarTooltip } text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />
    },
    addresses: {
      icon: <CommunicationContacts />,
      label: 'Addressbook',
      route: '/addresses',
      value: 'address'
    },
    apps: {
      icon: <NavigationApps />,
      label: 'Applications',
      route: '/apps',
      value: 'app'
    },
    status: {
      icon: <ActionTrackChanges />,
      label: 'Status',
      renderLabel: this.renderStatusLabel,
      route: '/status',
      value: 'status'
    },
    signer: {
      fixed: true,
      icon: <SignerIcon className={ styles.signerIcon } />,
      label: 'Signer',
      renderLabel: this.renderSignerLabel,
      route: '/signer',
      value: 'signer'
    }
  }
}
