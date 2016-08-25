import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatCoins, formatEth } from '../../format';

const { IdentityIcon } = window.parity.react;

export default class EventRefund extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  // "35000000000000000", "20000008"
  // "35000000000000000", "20000000"
  // "7C585087238000", "1312D00"
  // 0x5af36e3e000000000000000000000000000000000000000000000000007c5850872380000000000000000000000000000000000000000000000000000000000001312d00

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
        <div>Refund</div>
        <div>{ buyerIcon }{ buyer }</div>
        <div>refunded</div>
        <div>{ formatCoins(amount) }</div>
        <div>@ { formatEth(price) }</div>
      </div>
    );
  }
}
