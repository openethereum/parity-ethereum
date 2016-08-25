import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatCoins, formatEth } from '../../format';

const { IdentityIcon } = window.parity.react;

export default class EventRefund extends Component {
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
        <div>{ formatBlockNumber(blockNumber) }:</div>
        <div>Refund:</div>
        <div>{ buyerIcon }</div>
        <div>refunded</div>
        <div>{ formatCoins(amount) }</div>
        <div>@ { formatEth(price) }</div>
      </div>
    );
  }
}
