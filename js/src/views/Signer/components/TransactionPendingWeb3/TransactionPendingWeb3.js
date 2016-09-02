import React, { Component, PropTypes } from 'react';

import TransactionPending from '../TransactionPending';
import Web3Compositor from '../Web3Compositor';

class TransactionPendingWeb3 extends Component {

  static contextTypes = {
    web3: PropTypes.object.isRequired
  };

  static propTypes = {
    id: PropTypes.string.isRequired,
    from: PropTypes.string.isRequired,
    value: PropTypes.string.isRequired, // wei hex
    gasPrice: PropTypes.string.isRequired, // wei hex
    gas: PropTypes.string.isRequired, // hex
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    isSending: PropTypes.bool.isRequired,
    to: PropTypes.string, // undefined if it's a contract
    data: PropTypes.string, // hex
    nonce: PropTypes.number,
    className: PropTypes.string
  };

  state = {
    chain: 'homestead',
    fromBalance: null, // avoid required prop loading warning
    toBalance: null // avoid required prop loading warning in case there's a to address
  }

  render () {
    const { web3 } = this.context;
    const { fromBalance, toBalance, chain } = this.state;
    let { from, to } = this.props;

    from = web3.toChecksumAddress(from);
    to = to ? web3.toChecksumAddress(to) : to;

    return (
      <TransactionPending
        { ...this.props }
        from={ from }
        to={ to }
        fromBalance={ fromBalance }
        toBalance={ toBalance }
        chain={ chain }
      />
    );
  }

  // todo [adgo] - call next() only after all CBs are executed
  onTick (next) {
    this.fetchChain();
    this.fetchBalances(next);
  }

  fetchChain () {
    this.context.web3.ethcore.getNetChain((err, chain) => {
      if (err) {
        return console.warn('err fetching chain', err);
      }
      this.setState({ chain });
    });
  }

  fetchBalances (next) {
    const { from, to } = this.props;
    this.fetchBalance(from, 'from', next);

    if (!to) {
      return;
    }

    this.fetchBalance(to, 'to', next);
  }

  fetchBalance (address, owner, next) {
    this.context.web3.eth.getBalance(address, (err, balance) => {
      next(err);

      if (err) {
        console.warn('err fetching balance for ', address, err);
        return;
      }

      this.setState({
        [owner + 'Balance']: balance
      });
    });
  }

}

export default Web3Compositor(TransactionPendingWeb3);
