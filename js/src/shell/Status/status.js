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
import { connect } from 'react-redux';

import { BlockStatus } from '@parity/ui';

import Consensus from './Consensus';
import Upgrade from './Upgrade';

import styles from './status.css';

function Status ({ clientVersion, isTest, netChain, netPeers, upgradeStore }) {
  const [ clientName, , versionString, , ] = (clientVersion || '').split('/');
  const [ versionNumber, versionType, , versionDate ] = (versionString || '').split('-');

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
        <BlockStatus />
        <div className={ `${styles.network} ${styles[isTest ? 'test' : 'live']}` }>
          { netChain }
        </div>
        <div className={ styles.peers }>
          { netPeers.active.toFormat() }/{ netPeers.connected.toFormat() }/{ netPeers.max.toFormat() } peers
        </div>
      </div>
    </div>
  );
}

Status.propTypes = {
  clientVersion: PropTypes.string,
  isTest: PropTypes.bool,
  netChain: PropTypes.string,
  netPeers: PropTypes.object,
  upgradeStore: PropTypes.object.isRequired
};

function mapStateToProps (state) {
  const { clientVersion, netPeers, netChain, isTest } = state.nodeStatus;

  return {
    clientVersion,
    netPeers,
    netChain,
    isTest
  };
}

export default connect(
  mapStateToProps,
  null
)(Status);
