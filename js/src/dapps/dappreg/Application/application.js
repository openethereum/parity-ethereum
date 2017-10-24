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

import React, { Component } from 'react';
import { observer } from 'mobx-react';

import DappsStore from '../dappsStore';

import Dapps from '../Dapps';
import Transactions from '../Transactions';
import Warning from '../Warning';
import styles from './application.css';

@observer
export default class Application extends Component {
  dappsStore = DappsStore.get();

  render () {
    if (this.dappsStore.isLoading) {
      return (
        <div className={ styles.loading }>
          Loading application
        </div>
      );
    }

    const { ownDapps, otherDapps } = this.dappsStore;

    return (
      <div className={ styles.body }>
        <div className={ styles.header }>
          DAPP REGISTRY, a global view of decentralized applications available on the network. Putting the puzzle together.
        </div>

        <div>
          <Dapps
            dapps={ ownDapps }
            own
            title='My Dapps'
          />
          <Dapps
            dapps={ otherDapps }
            title='Other Dapps'
          />
        </div>

        <div className={ styles.footer }>
          { this.dappsStore.count } applications registered, { this.dappsStore.ownedCount } owned by user
        </div>

        <Transactions />
        <Warning />
      </div>
    );
  }
}
