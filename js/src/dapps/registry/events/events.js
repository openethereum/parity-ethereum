import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardActions, CardText } from 'material-ui/Card';
import Checkbox from 'material-ui/Checkbox';

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
          <Checkbox
            label='Reserved'
            checked={ subscriptions.Reserved !== null }
            disabled={ pending.Reserved }
            onCheck={ this.onReservedChanged }
            style={ inlineButton }
          />
          <Checkbox
            label='Dropped'
            checked={ subscriptions.Dropped !== null }
            disabled={ pending.Dropped }
            onCheck={ this.onDroppedChanged }
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

  onReservedChanged = (e, isChecked) => {
    const { pending, subscriptions, actions } = this.props;
    if (!pending.Reserved) {
      if (isChecked && subscriptions.Reserved === null) {
        actions.subscribe('Reserved');
      }
    }
  };
  onDroppedChanged = (e, isChecked) => {
    const { pending, subscriptions, actions } = this.props;
    if (!pending.Dropped) {
      if (isChecked && subscriptions.Dropped === null) {
        actions.subscribe('Dropped');
      }
    }
  };
}
