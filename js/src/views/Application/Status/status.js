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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import styles from './status.css';

class Status extends Component {
  static propTypes = {
    blockNumber: PropTypes.object,
    clientVersion: PropTypes.string,
    netPeers: PropTypes.object,
    netChain: PropTypes.string,
    syncing: PropTypes.oneOfType([
      PropTypes.bool,
      PropTypes.object
    ]),
    isTest: PropTypes.bool
  }

  render () {
    const { clientVersion, blockNumber, netChain, netPeers, syncing, isTest } = this.props;
    const netStyle = `${styles.network} ${styles[isTest ? 'networktest' : 'networklive']}`;
    let blockStatus = null;

    if (!blockNumber) {
      return null;
    }

    if (syncing) {
      if (!syncing.warpChunksAmount.eq(syncing.warpChunksProcessed)) {
        blockStatus = (
          <div className={ styles.syncing }>
            { syncing.warpChunksProcessed.mul(100).div(syncing.warpChunksAmount).toFormat(2) }% warp restore
          </div>
        );
      } else {
        let warpStatus = null;

        if (syncing.blockGap) {
          const [first, last] = syncing.blockGap;

          warpStatus = (
            <span>, { first.mul(100).div(last).toFormat(2) }% historic</span>
          );
        }

        blockStatus = (
          <div className={ styles.syncing }>
            <span>{ syncing.currentBlock.toFormat() }/{ syncing.highestBlock.toFormat() } syncing</span>
            { warpStatus }
          </div>
        );
      }
    } else {
      blockStatus = (
        <div className={ styles.block }>
          { blockNumber.toFormat() } best block
        </div>
      );
    }

    return (
      <div className={ styles.status }>
        <div className={ styles.version }>
          { clientVersion }
        </div>
        <div className={ styles.netinfo }>
          <div>
            { blockStatus }
            <div className={ styles.peers }>
              { netPeers.active.toFormat() }/{ netPeers.connected.toFormat() }/{ netPeers.max.toFormat() } peers
            </div>
          </div>
          <div className={ netStyle }>
            { isTest ? 'test' : netChain }
          </div>
        </div>
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { blockNumber, clientVersion, netPeers, netChain, syncing, isTest } = state.nodeStatus;

  return {
    blockNumber,
    clientVersion,
    netPeers,
    netChain,
    syncing,
    isTest
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Status);
