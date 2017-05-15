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

const TITLE_STYLE = {
  fontSize: '2em'
};

const inlineButton = {
  display: 'inline-block',
  width: 'auto',
  marginRight: '1em'
};

@observer
export default class Events extends Component {
  eventsStore = EventsStore.get();

  render () {
    const { shown } = this.eventsStore;

    return (
      <Card className={ styles.events }>
        <CardHeader
          title='Events'
          titleStyle={ TITLE_STYLE }
        />
        <CardActions className={ styles.options }>
          <Toggle
            label='Reservations'
            toggled={ shown.has('reservations') }
            onToggle={ this.handleReservationsToggle }
            style={ inlineButton }
          />
          <Toggle
            label='Metadata'
            toggled={ shown.has('metadata') }
            onToggle={ this.handleMetadataToggle }
            style={ inlineButton }
          />
          <Toggle
            label='Reverses'
            toggled={ shown.has('reverses') }
            onToggle={ this.handleReversesToggle }
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

    if (events.length === 0) {
      return (
        <div>
          No events to display
        </div>
      );
    }

    return events.map((event) => {
      return (
        <Event
          event={ event }
          key={ event.id }
        />
      );
    });
  }

  handleReservationsToggle = (e, toggled) => {
    return this.eventsStore.toggle('reservations', toggled);
  };

  handleMetadataToggle = (e, toggled) => {
    return this.eventsStore.toggle('metadata', toggled);
  };

  handleReversesToggle = (e, toggled) => {
    return this.eventsStore.toggle('reverses', toggled);
  };
}
