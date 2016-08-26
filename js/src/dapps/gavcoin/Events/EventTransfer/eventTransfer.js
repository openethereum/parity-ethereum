import React, { Component, PropTypes } from 'react';

import { formatCoins } from '../../format';
import ColumnAddress from '../ColumnAddress';
import ColumnBlockNumber from '../ColumnBlockNumber';

export default class EventTransfer extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { from, to, value } = event.params;
    const { blockNumber } = event;
    const cls = `event ${event.state} ${event.type.toLowerCase()}`;

    return (
      <tr className={ cls }>
        <ColumnBlockNumber
          blockNumber={ blockNumber } />
        <td className='type'>Transfer</td>
        <td className='gavvalue'>-{ formatCoins(value) }GAV</td>
        <td></td>
        <ColumnAddress
          address={ from } />
        <ColumnAddress
          address={ to } />
      </tr>
    );
  }
}
