import React, { Component, PropTypes } from 'react';

export default class Completed extends Component {
  static propTypes = {
    visible: PropTypes.bool.isRequired
  }

  render () {
    if (!this.props.visible) {
      return null;
    }

    return (
      <div>
        <p>Your node setup has been completed successfully.</p>
      </div>
    );
  }
}
