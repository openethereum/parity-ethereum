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

import React, { Component } from 'react';

import { loadAllTokens, subscribeEvents, unsubscribeEvents } from '../services';
import Container from '../Container';

import styles from './events.css';

export default class Deploy extends Component {
  state = {
    subscriptionId: 0,
    loading: true,
    events: [],
    pendingEvents: [],
    minedEvents: []
  }

  componentDidMount () {
    loadAllTokens()
      .then((tokens) => {
        const addresses = tokens.map((token) => token.address);

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

    return loading
      ? this.renderLoading()
      : this.renderEvents();
  }

  renderLoading () {
    return (
      <Container>
        <div className={ styles.statusHeader }>
          Attaching events
        </div>
      </Container>
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
      <Container>
        <div className={ styles.statusHeader }>
          There are currently no events available
        </div>
      </Container>
    );
  }

  renderEventsList () {
    return (
      <Container>events go here ...</Container>
    );
  }

  eventCallback = (error, events) => {
    if (error) {
      console.error('eventCallback', error);
      return;
    }

    console.log(events);
  }
}
