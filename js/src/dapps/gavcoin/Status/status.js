import React, { Component, PropTypes } from 'react';

export default class Status extends Component {
  static propTypes = {
    blockNumber: PropTypes.string,
    totalSupply: PropTypes.string,
    remaining: PropTypes.string,
    price: PropTypes.string
  }

  render () {
    if (!this.props.totalSupply) {
      return null;
    }

    return (
      <div>#{ this.props.blockNumber }: { this.props.remaining } coins remaining ({ this.props.totalSupply } total), price of { this.props.price }</div>
    );
  }
}
