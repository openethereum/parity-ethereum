import React, { Component } from 'react';

import TransactionFinished from '../../../TransactionFinished';
import styles from './TransactionFinishedPage.css';

import transactionsData from '../../transactions.data';

export default class TransactionFinishedPage extends Component {

  render () {
    return (
      <div>
        <h1>Transaction Finished</h1>
        <p>Transactions that got rejected / confirmed</p>
        { this.renderTransactionsFinished() }
      </div>
    );
  }

  renderTransactionsFinished () {
    return transactionsData.map(t => (
      <div className={ styles.componentContainer } key={ t.id }>
        <h4>{ t._desc }</h4>
        <TransactionFinished { ...t } />
        { this.renderInfo(t) }
      </div>
    ));
  }

  renderInfo (t) {
    return null;
  }
}
