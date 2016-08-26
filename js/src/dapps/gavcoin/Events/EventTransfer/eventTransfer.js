import React, { Component, PropTypes } from 'react';

import Event from '../Event';

export default class EventTransfer extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { from, to, value } = event.params;

    return (
      <Event
        event={ event }
        fromAddress={ from }
        toAddress={ to }
        value={ value } />
    );
  }
}
