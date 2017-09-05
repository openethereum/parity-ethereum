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

import React, { Component } from 'react';

import { api } from '../../parity';
import { loadAllTokens, subscribeEvents, unsubscribeEvents } from '../../services';
import Container from '../../Container';
import Event from '../Event';

import styles from '../../Deploy/Events/events.css';

export default class Events extends Component {
  state = {
    subscriptionId: 0,
    loading: true,
    events: [],
    pendingEvents: [],
    minedEvents: [],
    tokens: []
  }

  componentDidMount () {
    loadAllTokens()
      .then((tokens) => {
        const addresses = tokens.map((token) => token.address);

        this.setState({ tokens });
        return subscribeEvents(addresses, this.eventCallback);
      })
      .then((subscriptionId) => {
        this.setState({ subscriptionId, loading: false });
      })
      .catch((error) => {
        console.error('componentDidMount', error);
      });
  }

  componentWillUnmount () {
    const { subscriptionId } = this.state;

    if (subscriptionId) {
      unsubscribeEvents(subscriptionId);
    }
  }

  render () {
    const { loading } = this.state;

    return (
      <Container>
        { loading ? this.renderLoading() : this.renderEvents() }
      </Container>
    );
  }

  renderLoading () {
    return (
      <div className={ styles.statusHeader }>
        Attaching events
      </div>
    );
  }

  renderEvents () {
    const { events } = this.state;

    return events.length
      ? this.renderEventsList()
      : this.renderEventsNone();
  }

  renderEventsNone () {
    return (
      <div className={ styles.statusHeader }>
        There are currently no events available
      </div>
    );
  }

  renderEventsList () {
    const { events, tokens } = this.state;
    const rows = events.map((event) => {
      const token = tokens.find((token) => token.address === event.address);

      return (
        <Event
          key={ event.key }
          token={ token }
          event={ event }
        />
      );
    });

    return (
      <table className={ styles.eventList }>
        <tbody>
          { rows }
        </tbody>
      </table>
    );
  }

  logToEvent = (log) => {
    log.key = api.util.sha3(JSON.stringify(log));
    log.params = Object.keys(log.params).reduce((params, name) => {
      params[name] = log.params[name].value;
      return params;
    }, {});

    return log;
  }

  eventCallback = (error, logs) => {
    if (error) {
      console.error('eventCallback', error);
      return;
    }

    const { minedEvents, pendingEvents } = this.state;
    const minedNew = logs
      .filter((log) => log.type === 'mined')
      .map(this.logToEvent)
      .filter((log) => !minedEvents.find((event) => event.transactionHash === log.transactionHash))
      .reverse()
      .concat(minedEvents);
    const pendingNew = logs
      .filter((log) => log.type === 'pending')
      .map(this.logToEvent)
      .filter((log) => !pendingEvents.find((event) => event.transactionHash === log.transactionHash))
      .reverse()
      .concat(pendingEvents)
      .filter((log) => !minedNew.find((event) => event.transactionHash === log.transactionHash));
    const events = [].concat(pendingNew).concat(minedNew);

    this.setState({ loading: false, events, minedEvents: minedNew, pendingEvents: pendingNew });
  }
}
