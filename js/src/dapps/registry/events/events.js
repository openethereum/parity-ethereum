import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardActions, CardText } from 'material-ui/Card';
import Toggle from 'material-ui/Toggle';
import moment from 'moment';

import { bytesToHex, IdentityIcon } from '../parity.js';
import renderHash from '../ui/hash.js';
import renderAddress from '../ui/address.js';
import styles from './events.css';

const inlineButton = {
  display: 'inline-block',
  width: 'auto',
  marginRight: '1em'
};

const renderTimestamp = (ts) => {
  ts = moment(ts);
  return (
    <time dateTime={ ts.toISOString() }>
      <abbr title={ ts.format('MMMM Do YYYY, h:mm:ss a') }>{ ts.fromNow() }</abbr>
    </time>
  );
};

const renderReserved = (e, accounts, contacts) => (
  <p key={ e.key } className={ styles.reserved }>
    { renderAddress(e.parameters.owner, accounts, contacts) }
    { ' ' }
    <abbr title={ e.transaction }>reserved</abbr>
    { ' ' }
    <code>{ renderHash(bytesToHex(e.parameters.name)) }</code>
    { ' ' }
    { renderTimestamp(e.timestamp) }
  </p>
);

const renderDropped = (e, accounts, contacts) => (
  <p key={ e.key } className={ styles.dropped }>
    { renderAddress(e.parameters.owner, accounts, contacts) }
    { ' ' }
    <abbr title={ e.transaction }>dropped</abbr>
    { ' ' }
    <code>{ renderHash(bytesToHex(e.parameters.name)) }</code>
    { ' ' }
    { renderTimestamp(e.timestamp) }
  </p>
);

const eventTypes = {
  Reserved: renderReserved,
  Dropped: renderDropped
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

  static childContextTypes = { api: PropTypes.object.isRequired }
  getChildContext () {
    // TODO let /src/ui/IdentityIcon import from the api directly
    return { api: window.parity.api };
  }

  render () {
    const { subscriptions, pending, accounts, contacts } = this.props;
    return (
      <Card className={ styles.events }>
        <CardHeader title={ 'Stuff Happening' } />
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
        </CardActions>
        <CardText>{
          this.props.events
            .filter((e) => eventTypes[e.type])
            .map((e) => eventTypes[e.type](e, accounts, contacts))
        }</CardText>
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
}
