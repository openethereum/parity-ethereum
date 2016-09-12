import React, { Component, PropTypes } from 'react';

import { Container } from '../../../ui';

import Summary from '../Summary';
import styles from './list.css';

export default class List extends Component {
  static propTypes = {
    accounts: PropTypes.object,
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
    const keys = Object.keys(accounts || {});

    if (!keys.length) {
      return (
        <Container className={ styles.empty }>
          <div>
            There are currently no accounts or addresses to display.
          </div>
        </Container>
      );
    }

    return keys.map((address, idx) => {
      const account = accounts[address];

      return (
        <div
          className={ styles.account }
          key={ address }>
          <Summary
            contact={ contact }
            account={ account } />
        </div>
      );
    });
  }
}
