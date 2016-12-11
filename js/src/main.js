// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { Redirect, Router, Route, IndexRoute } from 'react-router';

import { Accounts, Account, Addresses, Address, Application, Contract, Contracts, WriteContract, Wallet, Dapp, Dapps, Settings, SettingsBackground, SettingsParity, SettingsProxy, SettingsViews, Signer, Status } from '~/views';

import styles from './reset.css';

export default class MainApplication extends Component {
  static propTypes = {
    routerHistory: PropTypes.any.isRequired
  };

  handleDeprecatedRoute = (nextState, replace) => {
    const { address } = nextState.params;
    const redirectMap = {
      account: 'accounts',
      address: 'addresses',
      contract: 'contracts'
    };

    const oldRoute = nextState.routes[0].path;
    const newRoute = Object.keys(redirectMap).reduce((newRoute, key) => {
      return newRoute.replace(new RegExp(`^/${key}`), '/' + redirectMap[key]);
    }, oldRoute);

    console.warn(`Route "${oldRoute}" is deprecated. Please use "${newRoute}"`);
    replace(newRoute.replace(':address', address));
  }

  render () {
    const { routerHistory } = this.props;

    return (
      <Router className={ styles.reset } history={ routerHistory }>
        <Redirect from='/' to='/accounts' />
        <Redirect from='/auth' to='/accounts' query={ {} } />
        <Redirect from='/settings' to='/settings/views' />

        { /** Backward Compatible links */ }
        <Route path='/account/:address' onEnter={ this.handleDeprecatedRoute } />
        <Route path='/address/:address' onEnter={ this.handleDeprecatedRoute } />
        <Route path='/contract/:address' onEnter={ this.handleDeprecatedRoute } />

        <Route path='/' component={ Application }>
          <Route path='accounts'>
            <IndexRoute component={ Accounts } />
            <Route path=':address' component={ Account } />
            <Route path='/wallet/:address' component={ Wallet } />
          </Route>

          <Route path='addresses'>
            <IndexRoute component={ Addresses } />
            <Route path=':address' component={ Address } />
          </Route>

          <Route path='apps' component={ Dapps } />
          <Route path='app/:id' component={ Dapp } />

          <Route path='contracts'>
            <IndexRoute component={ Contracts } />
            <Route path='develop' component={ WriteContract } />
            <Route path=':address' component={ Contract } />
          </Route>

          <Route path='settings' component={ Settings }>
            <Route path='background' component={ SettingsBackground } />
            <Route path='proxy' component={ SettingsProxy } />
            <Route path='views' component={ SettingsViews } />
            <Route path='parity' component={ SettingsParity } />
          </Route>

          <Route path='signer' component={ Signer } />

          <Route path='status'>
            <IndexRoute component={ Status } />
            <Route path=':subpage' component={ Status } />
          </Route>
        </Route>
      </Router>
    );
  }
}
