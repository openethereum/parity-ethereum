import React, { Component, PropTypes } from 'react';

import { Container } from '../../../ui';

import Summary from '../Summary';
import styles from './list.css';

export default class List extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    contact: PropTypes.bool
  };

  render () {
    return (
      <div className={ styles.list }>
        { this.renderAccounts() }
      </div>
    );
  }

  renderAccounts () {
    const { accounts, contact } = this.props;

    if (!accounts || !accounts.length) {
      return (
        <Container className={ styles.empty }>
          <div>
            There are currently no accounts or addresses to display.
          </div>
        </Container>
      );
    }

    return accounts.map((account, idx) => {
      return (
        <div
          className={ styles.account }
          key={ account.address }>
          <Summary
            contact={ contact }
            account={ account } />
        </div>
      );
    });
  }
}
