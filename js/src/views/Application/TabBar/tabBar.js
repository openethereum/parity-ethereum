import React, { Component, PropTypes } from 'react';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { Tabs, Tab } from 'material-ui/Tabs';
import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';
import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';
import NavigationApps from 'material-ui/svg-icons/navigation/apps';

import { Tooltip } from '../../../ui';

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
            label='accounts'
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
            label='address book'
            onActive={ this.onActivate } />
          <Tab
            className={ hash === 'app' ? styles.tabactive : '' }
            data-route='/apps'
            value='app'
            icon={ <NavigationApps /> }
            label='apps'
            onActive={ this.onActivate } />
          <Tab
            className={ hash === 'signer' ? styles.tabactive : '' }
            data-route='/signer'
            value='signer'
            icon={ <ActionFingerprint /> }
            label='signer'
            onActive={ this.onActivate } />
        </Tabs>
      </Toolbar>
    );
  }

  onActivate = (tab) => {
    const { router } = this.context;

    router.push(tab.props['data-route']);
  }
}
