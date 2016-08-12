import React, { Component, PropTypes } from 'react';

import { Tabs, Tab } from 'material-ui/Tabs';

import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import ActionDashboard from 'material-ui/svg-icons/action/dashboard';
import NavigationApps from 'material-ui/svg-icons/navigation/apps';

export default class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object
  }

  render () {
    return (
      <Tabs>
        <Tab
          data-route='/accounts'
          icon={ <ActionAccountBalanceWallet /> }
          label='accounts'
          onActive={ this.onActivate } />
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
    );
  }

  onActivate = (tab) => {
    this.context.router.push(tab.props['data-route']);
  }
}
