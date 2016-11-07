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
    events: PropTypes.array,
    loading: PropTypes.bool
  }

  componentDidMount () {
    const { address, subscribeToContractEvents } = this.props;
    subscribeToContractEvents(address);
  }

  shouldComponentUpdate (nextProps) {
    if (nextProps.loading && this.props.loading) {
      return false;
    }

    return true;
  }

  render () {
    const { events, loading } = this.props;

    if (!events || loading) {
      return (
        <Container className={ styles.eventsContainer }>
          <ContainerTitle title='events' />
          <LinearProgress mode='indeterminate' />
        </Container>
      );
    }

    if (events.length === 0) {
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
            events.map((event) => {
              return (
                <Event
                  event={ event }
                  key={ event.key }
                />
              );
            })
          }
          </tbody>
        </table>
      </Container>
    );
  }

}

function mapStateToProps (_, initProps) {
  const { address } = initProps;

  return (state) => {
    const { contracts } = state.blockchain;

    const contract = contracts[address];

    const loading = contract.eventsLoading;
    const events = [].concat(
      contract.events.mined,
      contract.events.pending
    );

    return {
      events,
      loading
    };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    subscribeToContractEvents
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Events);

