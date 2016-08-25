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
    const cls = `event ${event.state} ${event.type.toLowerCase()}`;

    const buyerIcon = (
      <IdentityIcon inline center address={ buyer } />
    );

    return (
      <tr className={ cls }>
        <td className='blocknumber'>{ formatBlockNumber(blockNumber) }</td>
        <td className='type'>Refund</td>
        <td className='ethvalue'>@{ formatEth(price) }ΞTH</td>
        <td className='gavvalue'>-{ formatCoins(amount) }GAV</td>
        <td className='account'>{ buyerIcon }<div>{ buyer }</div></td>
        <td></td>
      </tr>
    );
  }
}
