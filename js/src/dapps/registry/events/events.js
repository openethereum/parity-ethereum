import React, { Component, PropTypes } from 'react';
import {Card, CardHeader, CardText} from 'material-ui/Card';
import TextField from 'material-ui/TextField';

const { IdentityIcon } = window.parity.react;
import styles from './events.css';
import bytesToHex from '../../../api/util/bytes-array-to-hex';

const renderReserved = (e) => (
  <p key={ e.key } className={ styles.reserved }>
    <div className={ styles.owner }>
      <IdentityIcon inline center address={ e.parameters.owner } />
      <code>{ e.parameters.owner }</code>
    </div>
    { ' ' }
    <abbr title={ e.transaction }>registered</abbr>
    { ' ' }
    <code>{ bytesToHex(e.parameters.name) }</code>
  </p>
);

export default class Events extends Component {

  static propTypes = {
    actions: PropTypes.object,
    events: PropTypes.array
  }
  componentDidMount () {
    // TODO remove this
    this.props.actions.subscribe('Reserved', 0, 'latest');
  }

  render () {
    return (
      <Card className={ styles.events }>
        <CardHeader title="Stuff Happening" />
        <CardText>{
          this.props.events
            .filter((e) => e.state === 'mined')
            .map(renderReserved)
        }</CardText>
      </Card>
    );
  }
}
