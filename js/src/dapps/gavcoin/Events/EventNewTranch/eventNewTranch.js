import React, { Component, PropTypes } from 'react';

import Event from '../Event';

export default class EventNewTranch extends Component {
  static propTypes = {
    event: PropTypes.object
  }

  render () {
    const { event } = this.props;
    const { price } = event.params;

    return (
      <Event
        event={ event }
        price={ price } />
    );
  }
}
