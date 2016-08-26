import React, { Component, PropTypes } from 'react';

import { formatCoins, formatEth } from '../../format';
import ColumnAddress from '../ColumnAddress';
import ColumnBlockNumber from '../ColumnBlockNumber';

export default class EventBuyin extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { buyer, price, amount } = event.params;
    const { blockNumber } = event;
    const cls = `event ${event.state} ${event.type.toLowerCase()}`;

    return (
      <tr className={ cls }>
        <ColumnBlockNumber
          blockNumber={ blockNumber } />
        <td className='type'>Buyin</td>
        <td className='gavvalue'>+{ formatCoins(amount) }GAV</td>
        <td className='ethvalue'>{ formatEth(price) }ΞTH</td>
        <ColumnAddress
          address={ buyer } />
        <td></td>
      </tr>
    );
  }
}
