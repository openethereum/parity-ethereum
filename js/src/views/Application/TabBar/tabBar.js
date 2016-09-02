import React, { Component, PropTypes } from 'react';

import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { Tabs, Tab } from 'material-ui/Tabs';

import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';
import NavigationApps from 'material-ui/svg-icons/navigation/apps';

import { Tooltip } from '../../../ui';

import styles from '../application.css';
import imagesParitybar from '../../../images/paritybar.png';

export default class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  render () {
    const hash = (window.location.hash || '')
      .replace('#/', '').replace('accounts', 'account').replace('apps', 'app')
      .split('?')[0].split('/')[0];

    return (
      <Toolbar
        className={ styles.toolbar }>
        <ToolbarGroup>
          <img
            className={ styles.logo }
            src={ imagesParitybar } />
        </ToolbarGroup>
        <Tabs
          className={ styles.tabs }
          value={ hash }>
          <Tab
            data-route='/accounts'
            value='account'
            icon={ <ActionAccountBalanceWallet /> }
            label='accounts'
            onActive={ this.onActivate }>
            <Tooltip
              left='6%' top='65%'
              text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />
          </Tab>
          <Tab
            data-route='/apps'
            value='app'
            icon={ <NavigationApps /> }
            label='apps'
            onActive={ this.onActivate } />
          <Tab
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
