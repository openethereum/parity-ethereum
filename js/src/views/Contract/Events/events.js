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
import LinearProgress from 'material-ui/LinearProgress';

import { subscribeToContractEvents } from '../../../redux/providers/blockchainActions';
import { Container, ContainerTitle } from '../../../ui';

import Event from './Event';
import styles from '../contract.css';

class Events extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    subscribeToContractEvents: PropTypes.func.isRequired,
    address: PropTypes.string,
    contract: PropTypes.object,
    blocks: PropTypes.object,
    transactions: PropTypes.object,
    isTest: PropTypes.bool
  }

  componentDidMount () {
    const { address, subscribeToContractEvents } = this.props;
    subscribeToContractEvents(address);
  }

  render () {
    const { contract, blocks, transactions, isTest } = this.props;

    if (!contract) {
      return null;
    }

    const { events } = contract;

    if (!events || this.eventsLoading()) {
      return (
        <Container className={ styles.eventsContainer }>
          <ContainerTitle title='events' />
          <LinearProgress mode='indeterminate' />
        </Container>
      );
    }

    const allEvents = [].concat(events.pending, events.mined);

    if (allEvents.length === 0) {
      return (
        <Container className={ styles.eventsContainer }>
          <ContainerTitle title='events' />
          <p>
            There are no events associated with this account
          </p>
        </Container>
      );
    }

    return (
      <Container className={ styles.eventsContainer }>
        <ContainerTitle title='events' />
        <table className={ styles.events }>
          <tbody>
          {
            allEvents.map((event) => {
              const block = blocks[event.blockNumber.toString()];
              const transaction = transactions[event.transactionHash] || {};

              return (
                <Event
                  event={ event }
                  key={ event.key }
                  block={ block }
                  transaction={ transaction }
                  isTest={ isTest }
                />
              );
            })
          }
          </tbody>
        </table>
      </Container>
    );
  }

  eventsLoading () {
    const { contract, blocks, transactions } = this.props;
    const { events } = contract;

    if (events.loading) {
      return true;
    }

    const allEvents = [].concat(events.pending, events.mined);

    const blockNumbers = allEvents.map(e => e.blockNumber.toString());
    const txHashes = allEvents.map(e => e.transactionHash);

    const pendingBlocks = blockNumbers
      .map(k => blocks[k])
      .filter(b => (b && b.pending) || !b);

    const pendingTransactions = txHashes
      .map(k => transactions[k])
      .filter(t => (t && t.pending) || !t);

    return pendingBlocks.length + pendingTransactions.length > 0;
  }
}

function mapStateToProps (state, ownProps) {
  const { isTest } = state.nodeStatus;
  const { blocks, transactions, contracts } = state.blockchain;

  const contract = contracts[ownProps.address];

  return {
    isTest,
    blocks,
    transactions,
    contract
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    subscribeToContractEvents
    // dispatch(subscribeToContractEvents(address, instance));
    // dispatch(subscribeToContractQueries(address, instance));
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Events);

