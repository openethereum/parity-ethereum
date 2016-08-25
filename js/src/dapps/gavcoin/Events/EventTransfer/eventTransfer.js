import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatCoins } from '../../format';

const { IdentityIcon } = window.parity.react;

export default class EventTransfer extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { from, to, value } = event.params;
    const { blockNumber } = event;
    const cls = `event ${event.state} ${event.type.toLowerCase()}`;

    const fromIcon = (
      <IdentityIcon inline center address={ from } />
    );
    const toIcon = (
      <IdentityIcon inline center address={ to } />
    );

    return (
      <tr className={ cls }>
        <td className='blocknumber'>{ formatBlockNumber(blockNumber) }</td>
        <td className='type'>Transfer</td>
        <td></td>
        <td className='gavvalue'>-{ formatCoins(value) }GAV</td>
        <td className='account'>{ fromIcon }{ from }</td>
        <td className='account'>{ toIcon }{ to }</td>
      </tr>
    );
  }
}
