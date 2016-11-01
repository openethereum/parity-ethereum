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
import { isEqual, uniq } from 'lodash';
import Snackbar from 'material-ui/Snackbar';
import { darkBlack } from 'material-ui/styles/colors';

import styles from './status.css';

class Status extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object,
    blockNumber: PropTypes.object,
    clientVersion: PropTypes.string,
    netPeers: PropTypes.object,
    netChain: PropTypes.string,
    isTest: PropTypes.bool,
    newTransactions: PropTypes.array
  }

  state = {
    newTransactions: [],
    notifyNewTx: false,
    newTxMessage: ''
  }

  componentWillReceiveProps (nextProps) {
    const nextTransactions = nextProps.newTransactions;
    const currTransactions = this.props.newTransactions;

    const nextTxHashes = nextTransactions.map(t => t.hash).sort();
    const currTxHashes = currTransactions.map(t => t.hash).sort();

    // If new transactions received, add them to the
    // `newTransactions` array in the current state
    if (!isEqual(nextTxHashes, currTxHashes)) {
      const stateTransactions = this.state.newTransactions;
      const stateTxHashes = stateTransactions.map(t => t.hash);

      const hashes = uniq([].concat(nextTxHashes, stateTxHashes));
      const transactions = []
        .concat(stateTransactions, nextTransactions)
        .filter(t => hashes.indexOf(t.hash) >= 0);

      this.setNewTransactions(transactions);
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
    const { newTransactions, notifyNewTx, newTxMessage } = this.state;

    let onClose = () => {};
    let onClick = () => {};

    if (newTransactions.length > 0) {
      const transaction = newTransactions[0];

      onClose = (reason) => {
        if (reason === 'clickaway') {
          return;
        }

        const transactions = this.state
          .newTransactions
          .filter(t => t.hash !== transaction.hash);

        this.setNewTransactions(transactions);
      };

      onClick = () => {
        const { accounts } = this.props;
        const { router } = this.context;

        const transaction = newTransactions[0];
        const account = accounts[transaction.to]
          ? accounts[transaction.to]
          : accounts[transaction.from];

        if (!account) {
          return;
        }

        const viewLink = `/account/${account.address}`;
        router.push(viewLink);
        onClose();
      };
    }

    return (
      <Snackbar
        action='View'
        open={ notifyNewTx }
        message={ newTxMessage }
        autoHideDuration={ 6000 }
        onRequestClose={ onClose }
        onActionTouchTap={ onClick }
        bodyStyle={ {
          backgroundColor: darkBlack
        } }
      />
    );
  }

  setNewTransactions (newTransactions) {
    if (newTransactions.length === 0) {
      this.setState({
        notifyNewTx: false,
        newTransactions
      });

      return;
    }

    const { accounts } = this.props;

    const transaction = newTransactions[0];
    const account = accounts[transaction.to]
      ? accounts[transaction.to]
      : accounts[transaction.from];

    const newTxMessage = account
      ? (<span>
        A new transaction to
        <span className={ styles.accountName }> { account.name } </span>
        has been received.
      </span>)
      : '';

    this.setState({
      notifyNewTx: !!newTxMessage,
      newTxMessage,
      newTransactions
    });
  }
}

function mapStateToProps (state) {
  const { blockNumber, clientVersion, netPeers, netChain, isTest, newTransactions } = state.nodeStatus;
  const { accounts } = state.personal;

  return {
    accounts,
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
