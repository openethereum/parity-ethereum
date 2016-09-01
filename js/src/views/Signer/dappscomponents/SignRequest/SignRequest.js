import React, { Component, PropTypes } from 'react';

import Account from '../Account';
import TransactionPendingForm from '../TransactionPendingForm';
import TxHashLink from '../TxHashLink';

import styles from './SignRequest.css';

export default class SignRequest extends Component {

  // TODO [todr] re-use proptypes?
  static propTypes = {
    id: PropTypes.string.isRequired,
    address: PropTypes.string.isRequired,
    hash: PropTypes.string.isRequired,
    isFinished: PropTypes.bool.isRequired,
    chain: PropTypes.string.isRequired,
    balance: PropTypes.object,
    isSending: PropTypes.bool,
    onConfirm: PropTypes.func,
    onReject: PropTypes.func,
    status: PropTypes.string,
    className: PropTypes.string
  };

  render () {
    const className = this.props.className || '';
    return (
      <div className={ `${styles.container} ${className}` }>
        { this.renderDetails() }
        { this.renderActions() }
      </div>
    );
  }

  renderDetails () {
    const { address, balance, chain, hash } = this.props;

    return (
      <div className={ styles.signDetails }>
        <div className={ styles.address }>
          <Account address={ address } balance={ balance } chain={ chain } />
        </div>
        <div className={ styles.info } title={ hash }>
          <p>Dapp is requesting to sign arbitrary transaction using this account.</p>
          <p><strong>Confirm the transaction only if you trust the app.</strong></p>
        </div>
      </div>
    );
  }

  renderActions () {
    const { isFinished, status } = this.props;

    if (isFinished) {
      if (status === 'confirmed') {
        const { chain, hash } = this.props;

        return (
          <div className={ styles.actions }>
            <span className={ styles.isConfirmed }>Confirmed</span>
            <div>
              Transaction hash: <br />
              <TxHashLink chain={ chain } txHash={ hash } className={ styles.txHash } />
            </div>
          </div>
        );
      }

      return (
        <div className={ styles.actions }>
          <span className={ styles.isRejected }>Rejected</span>
        </div>
      );
    }

    return (
      <TransactionPendingForm
        isSending={ this.props.isSending }
        onConfirm={ this.onConfirm }
        onReject={ this.onReject }
        className={ styles.actions }
        />
    );
  }

  onConfirm = password => {
    const { id } = this.props;
    this.props.onConfirm({ id, password });
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }

}
