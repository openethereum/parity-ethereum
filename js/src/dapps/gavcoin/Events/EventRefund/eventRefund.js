import React, { Component, PropTypes } from 'react';

import { formatCoins, formatEth } from '../../format';
import ColumnAddress from '../ColumnAddress';
import ColumnBlockNumber from '../ColumnBlockNumber';

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

    return (
      <tr className={ cls }>
        <ColumnBlockNumber
          blockNumber={ blockNumber } />
        <td className='type'>Refund</td>
        <td className='gavvalue'>-{ formatCoins(amount) }GAV</td>
        <td className='ethvalue'>{ formatEth(price) }ΞTH</td>
        <ColumnAddress
          address={ buyer } />
        <td></td>
      </tr>
    );
  }
}
