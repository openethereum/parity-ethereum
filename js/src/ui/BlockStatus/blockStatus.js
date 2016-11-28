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

import styles from './blockStatus.css';

class BlockStatus extends Component {
  static propTypes = {
    blockNumber: PropTypes.object,
    syncing: PropTypes.oneOfType([
      PropTypes.bool,
      PropTypes.object
    ])
  }

  render () {
    const { blockNumber, syncing } = this.props;

    if (!blockNumber) {
      return null;
    }

    if (!syncing) {
      return (
        <div className={ styles.blockNumber }>
          { blockNumber.toFormat() } best block
        </div>
      );
    }

    if (syncing.warpChunksAmount && syncing.warpChunksProcessed && !syncing.warpChunksAmount.eq(syncing.warpChunksProcessed)) {
      return (
        <div className={ styles.syncStatus }>
          { syncing.warpChunksProcessed.mul(100).div(syncing.warpChunksAmount).toFormat(2) }% warp restore
        </div>
      );
    }

    let warpStatus = null;

    if (syncing.blockGap) {
      const [first, last] = syncing.blockGap;

      warpStatus = (
        <span>, { first.mul(100).div(last).toFormat(2) }% historic</span>
      );
    }

    return (
      <div className={ styles.syncStatus }>
        <span>{ syncing.currentBlock.toFormat() }/{ syncing.highestBlock.toFormat() } syncing</span>
        { warpStatus }
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { blockNumber, syncing } = state.nodeStatus;

  return {
    blockNumber,
    syncing
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(BlockStatus);
