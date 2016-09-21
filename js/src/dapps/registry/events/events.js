import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardActions, CardText } from 'material-ui/Card';
import Toggle from 'material-ui/Toggle';
import moment from 'moment';

import { bytesToHex } from '../parity.js';
import renderHash from '../ui/hash.js';
import renderAddress from '../ui/address.js';
import styles from './events.css';

const inlineButton = {
  display: 'inline-block',
  width: 'auto',
  marginRight: '1em'
};

const renderEvent = (classNames, verb) => (e, accounts, contacts) => {
  if (e.state === 'pending') {
    classNames += ' ' + styles.pending;
  }

  const timestamp = moment(e.timestamp);
  let status;
  if (e.state === 'pending') {
    status = (<abbr title='This transaction has not been synced with the network yet.'>pending</abbr>);
  } else {
    status = (
      <time dateTime={ timestamp.toISOString() }>
        <abbr title={ timestamp.format('MMMM Do YYYY, h:mm:ss a') }>{ timestamp.fromNow() }</abbr>
      </time>
    );
  }

  return (
    <div key={ e.key } className={ classNames }>
      { renderAddress(e.parameters.owner, accounts, contacts) }
      { ' ' }<abbr title={ e.transaction }>{ verb }</abbr>
      { ' ' }<code>{ renderHash(bytesToHex(e.parameters.name)) }</code>
      { ' ' }{ status }
    </div>
  );
};

const eventTypes = {
  Reserved: renderEvent(styles.reserved, 'reserved'),
  Dropped: renderEvent(styles.dropped, 'dropped')
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
