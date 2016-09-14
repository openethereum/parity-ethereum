import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardActions, CardText } from 'material-ui/Card';
import Toggle from 'material-ui/Toggle';

import { IdentityIcon } from '../parity.js';
import styles from './events.css';
import bytesToHex from '../../../api/util/bytes-array-to-hex';

const inlineButton = {
  display: 'inline-block',
  width: 'auto',
  marginRight: '1em'
};

const renderOwner = (owner) => (
  <span className={ styles.owner }>
    <IdentityIcon inline center address={ owner } />
    <code>{ owner }</code>
  </span>
);

const renderReserved = (e) => (
  <p key={ e.key } className={ styles.reserved }>
    { renderOwner(e.parameters.owner) }
    { ' ' }
    <abbr title={ e.transaction }>reserved</abbr>
    { ' ' }
    <code>{ bytesToHex(e.parameters.name) }</code>
  </p>
);

const renderDropped = (e) => (
  <p key={ e.key } className={ styles.dropped }>
    { renderOwner(e.parameters.owner) }
    { ' ' }
    <abbr title={ e.transaction }>dropped</abbr>
    { ' ' }
    <code>{ bytesToHex(e.parameters.name) }</code>
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
    events: PropTypes.array.isRequired
  }

  render () {
    const { subscriptions, pending } = this.props;
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
            .map((e) => eventTypes[e.type](e))
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
