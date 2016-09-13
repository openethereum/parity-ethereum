import React, { Component, PropTypes } from 'react';
import {Card, CardHeader, CardText} from 'material-ui/Card';
import TextField from 'material-ui/TextField';

const { IdentityIcon } = window.parity.react;
import styles from './events.css';
import bytesToHex from '../../../api/util/bytes-array-to-hex';

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
    actions: PropTypes.object,
    events: PropTypes.array
  }
  componentDidMount () {
    // TODO remove this
    this.props.actions.subscribe('Reserved', 0, 'latest');
    this.props.actions.subscribe('Dropped', 0, 'latest');
  }

  render () {
    return (
      <Card className={ styles.events }>
        <CardHeader title="Stuff Happening" />
        <CardText>{
          this.props.events
            .filter((e) => eventTypes[e.type])
            .map((e) => eventTypes[e.type](e))
        }</CardText>
      </Card>
    );
  }
}
