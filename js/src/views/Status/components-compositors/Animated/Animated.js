
import React, { Component } from 'react';
import AnimateChildren from './children';

export default Wrapped => class Animated extends Component {
  render () {
    return (
      <AnimateChildren>
        <Wrapped { ...this.props } />
      </AnimateChildren>
    );
  }
};
