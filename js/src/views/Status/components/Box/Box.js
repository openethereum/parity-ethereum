
import React, { Component, PropTypes } from 'react';

export default class Box extends Component {

  renderValue () {
    if (!this.props.value) {
      return;
    }

    return (
      <h1>{ this.props.value }</h1>
    );
  }

  render () {
    return (
      <div className='dapp-box'>
        <h2>{ this.props.title }</h2>
        { this.renderValue() }
        { this.props.children }
      </div>
    );
  }

  static propTypes = {
    title: PropTypes.string.isRequired,
    value: PropTypes.string,
    children: PropTypes.element
  }

}
