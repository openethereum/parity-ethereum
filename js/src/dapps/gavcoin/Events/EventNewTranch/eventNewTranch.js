import React, { Component, PropTypes } from 'react';

import { formatEth } from '../../format';
import ColumnBlockNumber from '../ColumnBlockNumber';

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
        <ColumnBlockNumber
          blockNumber={ blockNumber } />
        <td className='type'>New Tranch</td>
        <td></td>
        <td className='ethvalue'>{ formatEth(price) }ΞTH</td>
        <td></td>
        <td></td>
      </tr>
    );
  }
}
