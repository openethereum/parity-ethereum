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
import React, { Component } from 'react';
import { Card, CardHeader, CardActions, CardText } from 'material-ui/Card';
import Toggle from 'material-ui/Toggle';

import Event from './event';
import EventsStore from './events.store';

import styles from './events.css';

const inlineButton = {
  display: 'inline-block',
  width: 'auto',
  marginRight: '1em'
};

@observer
export default class Events extends Component {
  eventsStore = EventsStore.get();

  render () {
    const { subscriptions } = this.eventsStore;
    const isReverseToggle = ['ReverseProposed', 'ReverseConfirmed', 'ReverseRemoved']
      .findIndex((key) => !!subscriptions.has(key)) > -1;

    return (
      <Card className={ styles.events }>
        <CardHeader title='Event Log' />
        <CardActions className={ styles.options }>
          <Toggle
            label='Reserved'
            toggled={ subscriptions.has('Reserved') }
            onToggle={ this.handleReservedToggle }
            style={ inlineButton }
          />
          <Toggle
            label='Dropped'
            toggled={ subscriptions.has('Dropped') }
            onToggle={ this.handleDroppedToggle }
            style={ inlineButton }
          />
          <Toggle
            label='DataChanged'
            toggled={ subscriptions.has('DataChanged') }
            onToggle={ this.handleDataChangedToggle }
            style={ inlineButton }
          />
          <Toggle
            label='Reverse Lookup'
            toggled={ isReverseToggle }
            onToggle={ this.handleReverseToggle }
            style={ inlineButton }
          />
        </CardActions>
        <CardText>
          <div className={ styles.eventsList }>
            { this.renderEvents() }
          </div>
        </CardText>
      </Card>
    );
  }

  renderEvents () {
    const { events } = this.eventsStore;

    return events.map((event) => {
      return (
        <Event
          event={ event }
          key={ event.id }
        />
      );
    });
  }

  toggleSubscriptions (key, isToggled) {
    if (isToggled) {
      this.eventsStore.subscribe(key);
    } else {
      this.eventsStore.unsubscribe(key);
    }
  }

  handleReservedToggle = (e, isToggled) => {
    return this.toggleSubscriptions('Reserved', isToggled);
  };

  handleDroppedToggle = (e, isToggled) => {
    return this.toggleSubscriptions('Dropped', isToggled);
  };

  handleDataChangedToggle = (e, isToggled) => {
    return this.toggleSubscriptions('DataChanged', isToggled);
  };

  handleReverseToggle = (e, isToggled) => {
    return this.toggleSubscriptions(['ReverseProposed', 'ReverseConfirmed', 'ReverseRemoved'], isToggled);
  };
}
