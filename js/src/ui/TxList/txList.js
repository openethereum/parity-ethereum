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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import Store from './store';
import TxRow from './TxRow';

import styles from './txList.css';

@observer
class TxList extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    address: PropTypes.string.isRequired,
    hashes: PropTypes.oneOfType([
      PropTypes.array,
      PropTypes.object
    ]).isRequired,
    netVersion: PropTypes.string.isRequired
  };

  store = new Store(this.context.api);

  componentWillMount () {
    this.store.loadTransactions(this.props.hashes);
  }

  componentWillUnmount () {
    this.store.unsubscribe();
  }

  componentWillReceiveProps (newProps) {
    this.store.loadTransactions(newProps.hashes);
  }

  render () {
    return (
      <table className={ styles.transactions }>
        <tbody>
          { this.renderRows() }
        </tbody>
      </table>
    );
  }

  renderRows () {
    const { address, netVersion } = this.props;

    return this.store.sortedHashes.map((txhash) => {
      const tx = this.store.transactions[txhash];
      const blockNumber = tx.blockNumber.toNumber();
      const block = this.store.blocks[blockNumber];

      return (
        <TxRow
          key={ tx.hash }
          tx={ tx }
          block={ block }
          address={ address }
          netVersion={ netVersion }
        />
      );
    });
  }
}

function mapStateToProps (state) {
  const { netVersion } = state.nodeStatus;

  return {
    netVersion
  };
}

export default connect(
  mapStateToProps,
  null
)(TxList);
