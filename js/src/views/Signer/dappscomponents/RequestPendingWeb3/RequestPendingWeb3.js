import React, { Component, PropTypes } from 'react';

import TransactionPendingWeb3 from '../TransactionPendingWeb3';
import SignWeb3 from '../SignRequestWeb3';
import Web3Compositor from '../Web3Compositor';

class RequestPendingWeb3 extends Component {

  static contextTypes = {
    web3: PropTypes.object.isRequired
  };

  static propTypes = {
    id: PropTypes.string.isRequired,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    isSending: PropTypes.bool.isRequired,
    payload: PropTypes.oneOfType([
      PropTypes.shape({ transaction: PropTypes.object.isRequired }),
      PropTypes.shape({ sign: PropTypes.object.isRequired })
    ]).isRequired,
    className: PropTypes.string
  };

  render () {
    const { payload, id, className, isSending, onConfirm, onReject } = this.props;

    if (payload.sign) {
      const { sign } = payload;
      return (
        <SignWeb3
          className={ className }
          onConfirm={ onConfirm }
          onReject={ onReject }
          isSending={ isSending }
          isFinished={ false }
          id={ id }
          address={ sign.address }
          hash={ sign.hash }
          />
      );
    }

    if (payload.transaction) {
      const { transaction } = payload;
      return (
        <TransactionPendingWeb3
          className={ className }
          onConfirm={ onConfirm }
          onReject={ onReject }
          isSending={ isSending }
          id={ id }
          gasPrice={ transaction.gasPrice }
          gas={ transaction.gas }
          data={ transaction.data }
          from={ transaction.from }
          to={ transaction.to }
          value={ transaction.value }
          />
      );
    }

    // Unknown payload
    return null;
  }
}

export default Web3Compositor(RequestPendingWeb3);
