import React, { Component, PropTypes } from 'react';

import Event from '../Event';

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

    return (
      <Event
        event={ event }
        fromAddress={ buyer }
        value={ amount }
        price={ price } />
    );
  }
}
