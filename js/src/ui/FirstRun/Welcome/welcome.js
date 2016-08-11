import React, { Component, PropTypes } from 'react';

const LOGO_STYLE = {
  float: 'right',
  width: '25%',
  height: 'auto'
};

export default class FirstRun extends Component {
  static propTypes = {
    visible: PropTypes.bool.isRequired
  }

  render () {
    if (!this.props.visible) {
      return null;
    }

    return (
      <div>
        <img
          src='images/ethcore-logo-white-square.png'
          alt='Ethcore Ltd.'
          style={ LOGO_STYLE } />
        <p>Welcome to <strong>Parity</strong>, the fastest and simplest way to run your node.</p>
        <p>The next few steps will guide you through the process of setting up you Parity instance and the associated account.</p>
        <p>Click <strong>Next</strong> to continue your journey.</p>
      </div>
    );
  }
}
