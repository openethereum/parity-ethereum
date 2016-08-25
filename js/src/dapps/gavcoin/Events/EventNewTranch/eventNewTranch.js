import React, { Component, PropTypes } from 'react';

import { formatBlockNumber } from '../format';

const { Api } = window.parity;

export default class EventNewTranch extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { price } = event.params;
    const blockNumber = formatBlockNumber(event);
    const cls = `event ${event.state}`;

    return (
      <div className={ cls }>
        { blockNumber }: NewTranch: { Api.format.fromWei(price).toFormat(3) }
      </div>
    );
  }
}
