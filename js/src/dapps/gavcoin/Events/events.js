import React, { Component, PropTypes } from 'react';

export default class Events extends Component {
  static contextTypes = {
    instance: PropTypes.object
  }

  render () {
    return (
      <div className='events'>
        Events to display here
      </div>
    );
  }
}
