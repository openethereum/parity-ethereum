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
    const cls = `event ${event.state}`;

    return (
      <div className={ cls }>
        <div>{ formatBlockNumber(blockNumber) }</div>
        <div>NewTranch</div>
        <div>{ formatEth(price) }</div>
      </div>
    );
  }
}
