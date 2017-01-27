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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { BlockStatus } from '~/ui';

import styles from './status.css';

class Status extends Component {
  static propTypes = {
    clientVersion: PropTypes.string,
    isTest: PropTypes.bool,
    netChain: PropTypes.string,
    netPeers: PropTypes.object,
    upgradeStore: PropTypes.object.isRequired
  }

  render () {
    const { clientVersion, isTest, netChain, netPeers } = this.props;

    return (
      <div className={ styles.status }>
        <div className={ styles.version }>
          { clientVersion }
        </div>
        <div className={ styles.upgrade }>
          { this.renderConsensus() }
          { this.renderUpgradeButton() }
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

  renderConsensus () {
    const { upgradeStore } = this.props;

    if (!upgradeStore || !upgradeStore.consensusCapability) {
      return null;
    }

    if (upgradeStore.consensusCapability === 'capable') {
      return (
        <div>
          <FormattedMessage
            id='application.status.consensus.capable'
            defaultMessage='Capable'
          />
        </div>
      );
    }

    if (upgradeStore.consensusCapability.capableUntil) {
      return (
        <div>
          <FormattedMessage
            id='application.status.consensus.capableUntil'
            defaultMessage='Capable until #{blockNumber}'
            values={ {
              blockNumber: upgradeStore.consensusCapability.capableUntil
            } }
          />
        </div>
      );
    }

    if (upgradeStore.consensusCapability.incapableSince) {
      return (
        <div>
          <FormattedMessage
            id='application.status.consensus.incapableSince'
            defaultMessage='Incapable since #{blockNumber}'
            values={ {
              blockNumber: upgradeStore.consensusCapability.incapableSince
            } }
          />
        </div>
      );
    }

    return (
      <div>
        <FormattedMessage
          id='application.status.consensus.unknown'
          defaultMessage='Unknown capability'
        />
      </div>
    );
  }

  renderUpgradeButton () {
    const { upgradeStore } = this.props;

    if (!upgradeStore.available) {
      return null;
    }

    return (
      <div>
        <a
          href='javascript:void(0)'
          onClick={ upgradeStore.openModal }
        >
          <FormattedMessage
            id='application.status.upgrade'
            defaultMessage='Upgrade'
          />
        </a>
      </div>
    );
  }
}

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
