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
import { FormattedMessage } from 'react-intl';

import { BlockNumber, ClientVersion, NetChain, NetPeers, StatusIndicator } from '@parity/ui';

import Consensus from './Consensus';
import Upgrade from './Upgrade';

import styles from './status.css';

function Status ({ className = '', upgradeStore }, { api }) {
  return (
    <div className={ [styles.status, className].join(' ') }>
      <ClientVersion className={ styles.version } />
      <div className={ styles.upgrade }>
        <Consensus upgradeStore={ upgradeStore } />
        <Upgrade upgradeStore={ upgradeStore } />
      </div>
      <div className={ styles.netinfo }>
        <StatusIndicator id='application.status.health' />
        <BlockNumber
          className={ styles.blockNumber }
          message={
            <FormattedMessage
              id='ui.blockStatus.bestBlock'
              defaultMessage=' best block'
            />
          }
        />
        <NetPeers
          className={ styles.peers }
          message={
            <FormattedMessage
              id='ui.netPeers.peers'
              defaultMessage=' peers'
            />
          }
        />
        <NetChain />
      </div>
    </div>
  );
}

Status.contextTypes = {
  api: PropTypes.object.isRequired
};

Status.propTypes = {
  className: PropTypes.string,
  upgradeStore: PropTypes.object.isRequired
};

export default observer(Status);
