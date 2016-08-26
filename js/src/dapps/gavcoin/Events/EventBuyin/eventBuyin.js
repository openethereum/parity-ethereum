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
    const cls = `event ${event.state} ${event.type.toLowerCase()}`;

    const buyerIcon = (
      <IdentityIcon inline center address={ buyer } />
    );

    return (
      <tr className={ cls }>
        <td className='blocknumber'>{ formatBlockNumber(blockNumber) }</td>
        <td className='type'>Buyin</td>
        <td className='gavvalue'>+{ formatCoins(amount) }GAV</td>
        <td className='ethvalue'>@{ formatEth(price) }ΞTH</td>
        <td className='account'>{ buyerIcon }<div>{ buyer }</div></td>
        <td></td>
      </tr>
    );
  }
}
