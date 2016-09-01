import React, { Component, PropTypes } from 'react';

import logger from '../util/logger';
import Account from '../Account';
import Web3Compositor from '../Web3Compositor';

class AccountWeb3 extends Component {

  static contextTypes = {
    web3: PropTypes.object.isRequired
  };

  static propTypes = {
    address: PropTypes.string.isRequired,
    chain: PropTypes.string.isRequired,
    className: PropTypes.object,
    name: PropTypes.string
  }

  state = {
    balance: null
  };

  // from Web3Compositor
  onTick (next) {
    this.fetchBalance(next);
  }

  fetchBalance (next) {
    const { address } = this.props;
    this.context.web3.eth.getBalance(address, (err, balance) => {
      next(err);

      if (err) {
        logger.warn('err fetching balance for ', address, err);
        return;
      }

      this.setState({
        balance
      });
    });
  }

  render () {
    const { balance } = this.state;
    const { address, chain, className, name } = this.props;
    return (
      <Account
        balance={ balance }
        chain={ chain }
        address={ address }
        className={ className }
        name={ name }
      />
    );
  }

}

export default Web3Compositor(AccountWeb3);
