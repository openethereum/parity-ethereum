import React, { Component, PropTypes } from 'react';

import { formatBlockNumber } from '../format';

const { Api } = window.parity;

export default class EventRefund extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const blockNumber = formatBlockNumber(event);
    const cls = `event ${event.state}`;

    return (
      <div className={ cls }>
        { blockNumber }: Refund
      </div>
    );
  }
}
