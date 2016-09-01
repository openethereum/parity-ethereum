import React, { Component, PropTypes } from 'react';

import TransactionFinishedWeb3 from '../TransactionFinishedWeb3';
import SignWeb3 from '../SignRequestWeb3';
import Web3Compositor from '../Web3Compositor';

class RequestFinishedWeb3 extends Component {

  static contextTypes = {
    web3: PropTypes.object.isRequired
  };

  static propTypes = {
    id: PropTypes.string.isRequired,
    result: PropTypes.any.isRequired,
    payload: PropTypes.oneOfType([
      PropTypes.shape({ transaction: PropTypes.object.isRequired }),
      PropTypes.shape({ sign: PropTypes.object.isRequired })
    ]).isRequired,
    msg: PropTypes.string,
    status: PropTypes.string,
    error: PropTypes.string,
    className: PropTypes.string
  }

  render () {
    const { payload, id, result, msg, status, error, className } = this.props;

    if (payload.sign) {
      const { sign } = payload;
      return (
        <SignWeb3
          className={ className }
          isFinished
          id={ id }
          address={ sign.address }
          hash={ sign.hash }
          result={ result }
          msg={ msg }
          status={ status }
          error={ error }
          />
      );
    }

    if (payload.transaction) {
      const { transaction } = payload;
      return (
        <TransactionFinishedWeb3
          className={ className }
          txHash={ result }
          id={ id }
          gasPrice={ transaction.gasPrice }
          gas={ transaction.gas }
          from={ transaction.from }
          to={ transaction.to }
          value={ transaction.value }
          msg={ msg }
          status={ status }
          error={ error }
        />
      );
    }

    // Unknown payload
    return null;
  }
}

export default Web3Compositor(RequestFinishedWeb3);
