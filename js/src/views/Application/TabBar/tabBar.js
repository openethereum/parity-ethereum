import React, { Component, PropTypes } from 'react';

import Toolbar from 'material-ui/Toolbar';
import { Tabs, Tab } from 'material-ui/Tabs';

import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import ActionDashboard from 'material-ui/svg-icons/action/dashboard';
import NavigationApps from 'material-ui/svg-icons/navigation/apps';

import Tooltip from '../../../ui/Tooltip';

import styles from '../style.css';

export default class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object
  }

  render () {
    return (
      <Toolbar
        className={ styles.toolbar }>
        <img
          className={ styles.logo }
          src='images/parity-x56.png'
          alt='Parity' />
        <Tabs
          className={ styles.tabs }>
          <Tab
            data-route='/accounts'
            icon={ <ActionAccountBalanceWallet /> }
            label='accounts'
            onActive={ this.onActivate }>
            <Tooltip
              text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />
          </Tab>
          <Tab
            data-route='/tokens'
            icon={ <ActionDashboard /> }
            label='tokens'
            onActive={ this.onActivate } />
          <Tab
            data-route='/apps'
            icon={ <NavigationApps /> }
            label='apps'
            onActive={ this.onActivate } />
        </Tabs>
      </Toolbar>
    );
  }

  onActivate = (tab) => {
    this.context.router.push(tab.props['data-route']);
  }
}
