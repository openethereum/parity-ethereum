import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatCoins } from '../../format';

const { IdentityIcon } = window.parity.react;

export default class EventTransfer extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { from, to, value } = event.params;
    const { blockNumber } = event;
    const cls = `event ${event.state}`;

    const fromIcon = (
      <IdentityIcon inline center address={ from } />
    );
    const toIcon = (
      <IdentityIcon inline center address={ to } />
    );

    return (
      <div className={ cls }>
        <div>{ formatBlockNumber(blockNumber) }</div>
        <div>Transfer</div>
        <div>{ fromIcon }{ from }</div>
        <div>sent</div>
        <div>{ toIcon }{ to }</div>
        <div>{ formatCoins(value) }</div>
      </div>
    );
  }
}
