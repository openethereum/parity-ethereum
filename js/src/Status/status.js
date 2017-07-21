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

import React from 'react';
import PropTypes from 'prop-types';
import { observer } from 'mobx-react';

import { BlockStatus, Chain, StatusIndicator } from '@parity/ui';

import Consensus from './Consensus';
import Upgrade from './Upgrade';
import Store from './store';

import styles from './status.css';

function Status ({ upgradeStore }, { api }) {
  const store = Store.get(api);
  const [ clientName, , versionString, , ] = (store.clientVersion || '').split('/');
  const [ versionNumber, versionType, , versionDate ] = (versionString || '').split('-');
  const { connected, max } = store.netPeers;

  return (
    <div className={ styles.status }>
      <div className={ styles.version }>
        { clientName } { versionNumber }-{ versionDate } { versionType }
      </div>
      <div className={ styles.upgrade }>
        <Consensus upgradeStore={ upgradeStore } />
        <Upgrade upgradeStore={ upgradeStore } />
      </div>
      <div className={ styles.netinfo }>
        <StatusIndicator id='application.status.health' />
        <BlockStatus />
        <div className={ styles.peers }>
          { connected ? connected.toFormat() : '0' }/{ max ? max.toFormat() : '0' } peers
        </div>
        <Chain />
      </div>
    </div>
  );
}

Status.contextTypes = {
  api: PropTypes.object.isRequired
};

Status.propTypes = {
  upgradeStore: PropTypes.object.isRequired
};

export default observer(Status);
