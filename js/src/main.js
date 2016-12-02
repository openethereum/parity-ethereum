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
import { Redirect, Router, Route } from 'react-router';

import { Accounts, Account, Addresses, Address, Application, Contract, Contracts, WriteContract, Dapp, Dapps, Settings, SettingsBackground, SettingsParity, SettingsProxy, SettingsViews, Signer, Status } from 'views';

import styles from './reset.css';

export default class MainApplication extends Component {
  static propTypes = {
    routerHistory: PropTypes.any.isRequired
  };

  render () {
    const { routerHistory } = this.props;

    return (
      <Router className={ styles.reset } history={ routerHistory }>
        <Redirect from='/' to='/accounts' />
        <Redirect from='/auth' to='/accounts' query={ {} } />
        <Redirect from='/settings' to='/settings/views' />
        <Route path='/' component={ Application }>
          <Route path='accounts' component={ Accounts } />
          <Route path='account/:address' component={ Account } />
          <Route path='addresses' component={ Addresses } />
          <Route path='address/:address' component={ Address } />
          <Route path='apps' component={ Dapps } />
          <Route path='app/:id' component={ Dapp } />
          <Route path='contracts' component={ Contracts } />
          <Route path='contracts/write' component={ WriteContract } />
          <Route path='contract/:address' component={ Contract } />
          <Route path='settings' component={ Settings }>
            <Route path='background' component={ SettingsBackground } />
            <Route path='proxy' component={ SettingsProxy } />
            <Route path='views' component={ SettingsViews } />
            <Route path='parity' component={ SettingsParity } />
          </Route>
          <Route path='signer' component={ Signer } />
          <Route path='status' component={ Status } />
          <Route path='status/:subpage' component={ Status } />
        </Route>
      </Router>
    );
  }
}
