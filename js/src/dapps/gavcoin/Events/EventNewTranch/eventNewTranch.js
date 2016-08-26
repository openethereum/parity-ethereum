import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatEth } from '../../format';

export default class EventNewTranch extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { price } = event.params;
    const { blockNumber } = event;
    const cls = `event ${event.state} ${event.type.toLowerCase()}`;

    return (
      <tr className={ cls }>
        <td className='blocknumber'>{ formatBlockNumber(blockNumber) }</td>
        <td className='type'>New Tranch</td>
        <td></td>
        <td className='ethvalue'>{ formatEth(price) }ΞTH</td>
        <td></td>
        <td></td>
      </tr>
    );
  }
}
