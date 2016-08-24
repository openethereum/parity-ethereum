import React, { Component, PropTypes } from 'react';

const { Api } = window.parity;

const DIVISOR = 10 ** 6;

export default class EventBuyin extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const blockNumber = event.blockNumber;
    const { buyer, price, amount } = event.params;

    return (
      <div>
        #{ blockNumber.toFormat() }: Buyin: { buyer } bought { amount.div(DIVISOR).toFormat(3) } @ { Api.format.fromWei(price).toFormat(3) }
      </div>
    );
  }
}
