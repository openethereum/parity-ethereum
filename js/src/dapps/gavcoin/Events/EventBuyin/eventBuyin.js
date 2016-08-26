import React, { Component, PropTypes } from 'react';

import Event from '../Event';

export default class EventBuyin extends Component {
  static propTypes = {
    event: PropTypes.object
  }

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
