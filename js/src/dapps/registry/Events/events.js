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
import { Card, CardHeader, CardActions, CardText } from 'material-ui/Card';
import Toggle from 'material-ui/Toggle';
import moment from 'moment';

import { bytesToHex } from '../parity';
import renderHash from '../ui/hash';
import renderAddress from '../ui/address';
import styles from './events.css';

const inlineButton = {
  display: 'inline-block',
  width: 'auto',
  marginRight: '1em'
};

const renderStatus = (timestamp, isPending) => {
  timestamp = moment(timestamp);
  if (isPending) {
    return (<abbr title='This transaction has not been synced with the network yet.'>pending</abbr>);
  }
  return (
    <time dateTime={ timestamp.toISOString() }>
      <abbr title={ timestamp.format('MMMM Do YYYY, h:mm:ss a') }>{ timestamp.fromNow() }</abbr>
    </time>
  );
};

const renderEvent = (classNames, verb) => (e, accounts, contacts) => {
  const classes = e.state === 'pending'
    ? classNames + ' ' + styles.pending : classNames;

  return (
    <tr key={ e.key } className={ classes }>
      <td>{ renderAddress(e.parameters.owner.value, accounts, contacts) }</td>
      <td><abbr title={ e.transaction }>{ verb }</abbr></td>
      <td><code>{ renderHash(bytesToHex(e.parameters.name.value)) }</code></td>
      <td>{ renderStatus(e.timestamp, e.state === 'pending') }</td>
    </tr>
  );
};

const renderDataChanged = (e, accounts, contacts) => {
  let classNames = styles.dataChanged;
  if (e.state === 'pending') {
    classNames += ' ' + styles.pending;
  }

  return (
    <tr key={ e.key } className={ classNames }>
      <td>{ renderAddress(e.parameters.owner.value, accounts, contacts) }</td>
      <td><abbr title={ e.transaction }>updated</abbr></td>
      <td>
        key <code>{ new Buffer(e.parameters.plainKey.value).toString('utf8') }</code> of <code>{ renderHash(bytesToHex(e.parameters.name.value)) }</code>
      </td>
      <td>{ renderStatus(e.timestamp, e.state === 'pending') }</td>
    </tr>
  );
};

const eventTypes = {
  Reserved: renderEvent(styles.reserved, 'reserved'),
  Dropped: renderEvent(styles.dropped, 'dropped'),
  DataChanged: renderDataChanged
};

export default class Events extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    subscriptions: PropTypes.object.isRequired,
    pending: PropTypes.object.isRequired,
    events: PropTypes.array.isRequired,
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired
  }

  render () {
    const { subscriptions, pending, accounts, contacts } = this.props;

    const eventsObject = this.props.events
      .filter((e) => eventTypes[e.type])
      .reduce((eventsObject, event) => {
        const txHash = event.transaction;

        if (
          (eventsObject[txHash] && eventsObject[txHash].state === 'pending') ||
          !eventsObject[txHash]
        ) {
          eventsObject[txHash] = event;
        }

        return eventsObject;
      }, {});

    const events = Object
      .values(eventsObject)
      .sort((evA, evB) => {
        if (evA.state === 'pending') {
          return -1;
        }

        if (evB.state === 'pending') {
          return 1;
        }

        return evB.timestamp - evA.timestamp;
      })
      .map((e) => eventTypes[e.type](e, accounts, contacts));

    return (
      <Card className={ styles.events }>
        <CardHeader title='Event Log' />
        <CardActions className={ styles.options }>
          <Toggle
            label='Reserved'
            toggled={ subscriptions.Reserved !== null }
            disabled={ pending.Reserved }
            onToggle={ this.onReservedToggle }
            style={ inlineButton }
          />
          <Toggle
            label='Dropped'
            toggled={ subscriptions.Dropped !== null }
            disabled={ pending.Dropped }
            onToggle={ this.onDroppedToggle }
            style={ inlineButton }
          />
          <Toggle
            label='DataChanged'
            toggled={ subscriptions.DataChanged !== null }
            disabled={ pending.DataChanged }
            onToggle={ this.onDataChangedToggle }
            style={ inlineButton }
          />
        </CardActions>
        <CardText>
          <table className={ styles.eventsList }>
            <tbody>
              { events }
            </tbody>
          </table>
        </CardText>
      </Card>
    );
  }

  onReservedToggle = (e, isToggled) => {
    const { pending, subscriptions, actions } = this.props;
    if (!pending.Reserved) {
      if (isToggled && subscriptions.Reserved === null) {
        actions.subscribe('Reserved');
      } else if (!isToggled && subscriptions.Reserved !== null) {
        actions.unsubscribe('Reserved');
      }
    }
  };
  onDroppedToggle = (e, isToggled) => {
    const { pending, subscriptions, actions } = this.props;
    if (!pending.Dropped) {
      if (isToggled && subscriptions.Dropped === null) {
        actions.subscribe('Dropped');
      } else if (!isToggled && subscriptions.Dropped !== null) {
        actions.unsubscribe('Dropped');
      }
    }
  };
  onDataChangedToggle = (e, isToggled) => {
    const { pending, subscriptions, actions } = this.props;
    if (!pending.DataChanged) {
      if (isToggled && subscriptions.DataChanged === null) {
        actions.subscribe('DataChanged');
      } else if (!isToggled && subscriptions.DataChanged !== null) {
        actions.unsubscribe('DataChanged');
      }
    }
  };
}
