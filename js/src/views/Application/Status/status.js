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
import { isEqual } from 'lodash';
import Snackbar from 'material-ui/Snackbar';

import styles from './status.css';

class Status extends Component {
  static propTypes = {
    blockNumber: PropTypes.object,
    clientVersion: PropTypes.string,
    netPeers: PropTypes.object,
    netChain: PropTypes.string,
    isTest: PropTypes.bool,
    newTransactions: PropTypes.array
  }

  state = {
    newTransactions: []
  }

  componentWillReceiveProps (nextProps) {
    const { newTransactions } = nextProps;
    const oldNewTransactions = this.props.newTransactions;

    const newTxs = newTransactions.map(t => t.hash).sort();
    const oldTxs = oldNewTransactions.map(t => t.hash).sort();

    if (!isEqual(newTxs, oldTxs)) {
      const transactions = [].concat(newTransactions);

      if (transactions.length > 0) {
        transactions[0].open = true;
      }

      console.log('GOT TXS', transactions);

      this.setState({
        newTransactions: transactions
      });
    }
  }

  render () {
    const { clientVersion, blockNumber, netChain, netPeers, isTest } = this.props;

    const netStyle = `${styles.network} ${styles[isTest ? 'networktest' : 'networklive']}`;

    if (!blockNumber) {
      return null;
    }

    return (
      <div className={ styles.status }>
        { this.renderNewTransaction() }

        <div className={ styles.version }>
          { clientVersion }
        </div>
        <div className={ styles.netinfo }>
          <div>
            <div className={ styles.block }>
              { blockNumber.toFormat() } blocks
            </div>
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

  renderNewTransaction () {
    const { newTransactions } = this.state;

    return newTransactions.map((tx, index) => {
      const onClose = () => {
        const transactions = this.state
          .newTransactions
          .filter(t => t.hash !== tx.hash);

        if (transactions.length > 0) {
          transactions[0].open = true;
        }

        this.setState({ newTransactions: transactions });
      };

      return (
        <Snackbar
          key={ tx.hash }
          open={ tx.open || false }
          message={ `A new transaction has been detected: ${tx.hash}` }
          autoHideDuration={ 4000 }
          onRequestClose={ onClose }
        />
      );
    });
  }
}

function mapStateToProps (state) {
  const { blockNumber, clientVersion, netPeers, netChain, isTest, newTransactions } = state.nodeStatus;

  return {
    blockNumber,
    clientVersion,
    netPeers,
    netChain,
    isTest,
    newTransactions
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Status);
