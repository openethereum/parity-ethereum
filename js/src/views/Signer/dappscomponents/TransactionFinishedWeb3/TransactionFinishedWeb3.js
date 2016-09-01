import React, { Component, PropTypes } from 'react';

import TransactionFinished from '../TransactionFinished';
import Web3Compositor from '../Web3Compositor';

class TransactionFinishedWeb3 extends Component {

  static contextTypes = {
    web3: PropTypes.object.isRequired
  };

  static propTypes = {
    from: PropTypes.string.isRequired,
    to: PropTypes.string // undefined if it's a contract
  }

  state = {
    chain: 'homestead'
  };

  onTick (next) {
    this.fetchChain();
    this.fetchBalances(next);
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

  fetchChain () {
    this.context.web3.ethcore.getNetChain((err, chain) => {
      if (err) {
        return console.warn('err fetching chain', err);
      }

      this.setState({ chain });
    });
  }

  render () {
    const { fromBalance, toBalance, chain } = this.state;
    const { web3 } = this.context;

    let { from, to } = this.props;
    from = web3.toChecksumAddress(from);
    to = to ? web3.toChecksumAddress(to) : to;

    return (
      <TransactionFinished
        { ...this.props }
        from={ from }
        fromBalance={ fromBalance }
        to={ to }
        toBalance={ toBalance }
        chain={ chain }
        />
    );
  }
}

export default Web3Compositor(TransactionFinishedWeb3);
