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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Card, CardHeader, CardActions, CardText } from 'material-ui/Card';
import Toggle from 'material-ui/Toggle';
import moment from 'moment';

import { bytesToHex } from '../parity';
import Hash from '../ui/hash';
import Address from '../ui/address';

import { subscribe, unsubscribe } from './actions';
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

const renderEvent = (classNames, verb) => (e) => {
  const classes = e.state === 'pending'
    ? classNames + ' ' + styles.pending : classNames;

  return (
    <tr key={ e.key } className={ classes }>
      <td>
        <Address
          address={
            e.parameters.owner
              ? e.parameters.owner.value
              : e.from
          }
        />
      </td>
      <td>
        <abbr title={ e.transaction }>{ verb }</abbr>
      </td>
      <td>
        <code>
          <Hash hash={ bytesToHex(e.parameters.name.value) } />
        </code>
      </td>
      <td>
        { renderStatus(e.timestamp, e.state === 'pending') }
      </td>
    </tr>
  );
};

const renderDataChanged = (e) => {
  let classNames = styles.dataChanged;

  if (e.state === 'pending') {
    classNames += ' ' + styles.pending;
  }

  return (
    <tr key={ e.key } className={ classNames }>
      <td>
        <Address
          address={
            e.parameters.owner
              ? e.parameters.owner.value
              : e.from
          }
        />
      </td>
      <td>
        <abbr title={ e.transaction }>updated</abbr>
      </td>
      <td>
        key&nbsp;
        <code>
          { new Buffer(e.parameters.plainKey.value).toString('utf8') }
        </code>
        &nbsp;of&nbsp;
        <code>
          <Hash hash={ bytesToHex(e.parameters.name.value) } />
        </code>
      </td>
      <td>
        { renderStatus(e.timestamp, e.state === 'pending') }
      </td>
    </tr>
  );
};

const renderReverse = (e) => {
  const verb = ({
    ReverseProposed: 'proposed',
    ReverseConfirmed: 'confirmed',
    ReverseRemoved: 'removed'
  })[e.type];

  if (!verb) {
    return null;
  }

  const classes = [ styles.reverse ];

  if (e.state === 'pending') {
    classes.push(styles.pending);
  }

  // TODO: `name` is an indexed param, cannot display as plain text
  return (
    <tr key={ e.key } className={ classes.join(' ') }>
      <td>
        <Address address={ e.from } />
      </td>
      <td>{ verb }</td>
      <td>
        { 'name ' }
        <code key='name'>{ bytesToHex(e.parameters.name.value) }</code>
        { ' for ' }
        <Address key='reverse' address={ e.parameters.reverse.value } />
      </td>
      <td>
        { renderStatus(e.timestamp, e.state === 'pending') }
      </td>
    </tr>
  );
};

const eventTypes = {
  Reserved: renderEvent(styles.reserved, 'reserved'),
  Dropped: renderEvent(styles.dropped, 'dropped'),
  DataChanged: renderDataChanged,
  ReverseProposed: renderReverse,
  ReverseConfirmed: renderReverse,
  ReverseRemoved: renderReverse
};

class Events extends Component {
  static propTypes = {
    events: PropTypes.array.isRequired,
    pending: PropTypes.object.isRequired,
    subscriptions: PropTypes.object.isRequired,

    subscribe: PropTypes.func.isRequired,
    unsubscribe: PropTypes.func.isRequired
  }

  render () {
    const { subscriptions, pending } = this.props;

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
      .map((e) => eventTypes[e.type](e));

    const reverseToggled =
      subscriptions.ReverseProposed !== null &&
      subscriptions.ReverseConfirmed !== null &&
      subscriptions.ReverseRemoved !== null;
    const reverseDisabled =
      pending.ReverseProposed ||
      pending.ReverseConfirmed ||
      pending.ReverseRemoved;

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
          <Toggle
            label='Reverse Lookup'
            toggled={ reverseToggled }
            disabled={ reverseDisabled }
            onToggle={ this.onReverseToggle }
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
    const { pending, subscriptions, subscribe, unsubscribe } = this.props;

    if (!pending.Reserved) {
      if (isToggled && subscriptions.Reserved === null) {
        subscribe('Reserved');
      } else if (!isToggled && subscriptions.Reserved !== null) {
        unsubscribe('Reserved');
      }
    }
  };

  onDroppedToggle = (e, isToggled) => {
    const { pending, subscriptions, subscribe, unsubscribe } = this.props;

    if (!pending.Dropped) {
      if (isToggled && subscriptions.Dropped === null) {
        subscribe('Dropped');
      } else if (!isToggled && subscriptions.Dropped !== null) {
        unsubscribe('Dropped');
      }
    }
  };

  onDataChangedToggle = (e, isToggled) => {
    const { pending, subscriptions, subscribe, unsubscribe } = this.props;

    if (!pending.DataChanged) {
      if (isToggled && subscriptions.DataChanged === null) {
        subscribe('DataChanged');
      } else if (!isToggled && subscriptions.DataChanged !== null) {
        unsubscribe('DataChanged');
      }
    }
  };

  onReverseToggle = (e, isToggled) => {
    const { pending, subscriptions, subscribe, unsubscribe } = this.props;

    for (let e of ['ReverseProposed', 'ReverseConfirmed', 'ReverseRemoved']) {
      if (pending[e]) {
        continue;
      }

      if (isToggled && subscriptions[e] === null) {
        subscribe(e);
      } else if (!isToggled && subscriptions[e] !== null) {
        unsubscribe(e);
      }
    }
  };
}

const mapStateToProps = (state) => state.events;
const mapDispatchToProps = (dispatch) => bindActionCreators({ subscribe, unsubscribe }, dispatch);

export default connect(mapStateToProps, mapDispatchToProps)(Events);
