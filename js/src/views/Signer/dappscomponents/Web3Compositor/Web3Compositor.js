// no need for react since not using JSX
import React, { Component, PropTypes } from 'react';

export default Wrapped => class Web3Compositor extends Component {

  static contextTypes = {
    web3: PropTypes.object.isRequired
  };

  tickActive = false

  render () {
    return (
      <Wrapped { ...this.props } ref={ this.registerComponent } />
    );
  }

  componentDidMount () {
    this.tickActive = true;
    setTimeout(this.next);
  }

  componentWillUnmount () {
    this.tickActive = false;
  }

  next = () => {
    if (!this.tickActive) {
      return;
    }

    if (!this.wrapped || !this.wrapped.onTick) {
      setTimeout(this.next, 5000);
      return;
    }

    let nextCalled = false;
    this.wrapped.onTick(error => {
      if (nextCalled) {
        return;
      }
      nextCalled = true;
      setTimeout(this.next, error ? 10000 : 2000);
    });
  }

  registerComponent = component => {
    this.wrapped = component;
  }

};
