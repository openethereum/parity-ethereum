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
import { FormattedMessage } from 'react-intl';
import PropTypes from 'prop-types';
import { observer } from 'mobx-react';

import Store from './store';

import styles from './blockNumber.css';

function BlockNumber ({ className, message }, { api }) {
  const store = Store.get(api);

  if (!store.blockNumber) {
    return null;
  }

  if (!store.syncing) {
    return (
      <div className={ [styles.blockNumber, className].join(' ') }>
        { store.blockNumber.toFormat() }{ message }
      </div>
    );
  }

  if (store.syncing.warpChunksAmount && store.syncing.warpChunksProcessed && !store.syncing.warpChunksAmount.eq(store.syncing.warpChunksProcessed)) {
    return (
      <div className={ styles.syncStatus }>
        <FormattedMessage
          id='ui.blockStatus.warpRestore'
          defaultMessage='{percentage}% warp restore'
          values={ {
            percentage: store.syncing.warpChunksProcessed.mul(100).div(store.syncing.warpChunksAmount).toFormat(2)
          } }
        />
      </div>
    );
  }

  let syncStatus = null;
  let warpStatus = null;

  if (store.syncing.currentBlock && store.syncing.highestBlock) {
    syncStatus = (
      <span>
        <FormattedMessage
          id='ui.blockStatus.syncStatus'
          defaultMessage='{currentBlock}/{highestBlock} syncing'
          values={ {
            currentBlock: store.syncing.currentBlock.toFormat(),
            highestBlock: store.syncing.highestBlock.toFormat()
          } }
        />
      </span>
    );
  }

  if (store.syncing.blockGap) {
    const [first, last] = store.syncing.blockGap;

    warpStatus = (
      <span>
        <FormattedMessage
          id='ui.blockStatus.warpStatus'
          defaultMessage=', {percentage}% historic'
          values={ {
            percentage: first.mul(100).div(last).toFormat(2)
          } }
        />
      </span>
    );
  }

  return (
    <div className={ styles.syncStatus }>
      { syncStatus }
      { warpStatus }
    </div>
  );
}

BlockNumber.propTypes = {
  className: PropTypes.string,
  message: PropTypes.node
};

BlockNumber.contextTypes = {
  api: PropTypes.object.isRequired
};

const ObserverComponent = observer(BlockNumber);

ObserverComponent.Store = Store;

export default ObserverComponent;
