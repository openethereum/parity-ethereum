import React, { Component } from 'react';

import TransactionPendingWeb3 from '../../../TransactionPendingWeb3';
import styles from './TransactionPendingWeb3Page.css';

import transactionsData from '../../transactions.data';

export default class TransactionPendingWeb3Page extends Component {

  state = {};

  render () {
    return (
      <div>
        <h1>Transaction Pending</h1>
        <p>Transactions that are awaiting confirmaton / rejection </p>
        { this.renderTransactionsPending() }
      </div>
    );
  }

  renderTransactionsPending () {
    return transactionsData.map(t => (
      <div className={ styles.componentContainer } key={ t.id }>
        <h4>{ t._desc }</h4>
        <TransactionPendingWeb3
          { ...t }
          onConfirm={ this.onConfirm }
          onReject={ this.onReject }
          />
        { this.renderChosenAction(t.id) }
        { this.renderInfo(t) }
      </div>
    ));
  }

  renderInfo (t) {
    return null;
  }

  renderChosenAction (id) {
    const chosenAction = this.state['chosenAction_' + id];
    if (!chosenAction) {
      return;
    }

    return (
      <p>
        You have { chosenAction } this pending transaction
        { this.renderWithPassword(id) }.
      </p>
    );
  }

  // rejecting doens't require password
  renderWithPassword (id) {
    const password = this.state['chosenPassword_' + id];
    if (!password) {
      return;
    }

    return ' with password ' + password;
  }

  onConfirm = (id, password, gasPrice) => {
    this.setState({
      ['chosenAction_' + id]: 'confirmed',
      ['chosenPassword_' + id]: password
    });
  }

  onReject = id => {
    this.setState({
      ['chosenAction_' + id]: 'rejected',
      ['chosenPassword_' + id]: null
    });
  }

}
