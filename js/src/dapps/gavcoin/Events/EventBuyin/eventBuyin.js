import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatCoins, formatEth } from '../../format';

const { IdentityIcon } = window.parity.react;

export default class EventBuyin extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { buyer, price, amount } = event.params;
    const { blockNumber } = event;
    const cls = `event ${event.state}`;

    const buyerIcon = (
      <IdentityIcon inline center address={ buyer } />
    );

    return (
      <div className={ cls }>
        <div>{ formatBlockNumber(blockNumber) }</div>
        <div>Buyin</div>
        <div>{ buyerIcon }{ buyer }</div>
        <div>bought</div>
        <div>{ formatCoins(amount) }</div>
        <div>@</div>
        <div>{ formatEth(price) }</div>
      </div>
    );
  }
}
