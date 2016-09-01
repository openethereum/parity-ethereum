import React, { Component, PropTypes } from 'react';

import { getTxLink } from '../util/transaction';

export default class TxHashLink extends Component {

  static propTypes = {
    txHash: PropTypes.string.isRequired,
    chain: PropTypes.string.isRequired,
    children: PropTypes.node,
    className: PropTypes.string
  }

  state = {
    link: null
  };

  componentWillMount () {
    const { txHash, chain } = this.props;
    this.updateLink(txHash, chain);
  }

  componentWillReceiveProps (nextProps) {
    const { txHash, chain } = nextProps;
    this.updateLink(txHash, chain);
  }

  render () {
    const { children, txHash, className } = this.props;
    const { link } = this.state;

    return (
      <a
        href={ link }
        target='_blank'
        className={ className }>
        { children || txHash }
      </a>
    );
  }

  updateLink (txHash, chain) {
    const link = getTxLink(txHash, chain);
    this.setState({ link });
  }

}
