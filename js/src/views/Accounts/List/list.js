import React, { Component, PropTypes } from 'react';

import { Container } from '../../../ui';

import Summary from '../Summary';
import styles from './list.css';

export default class List extends Component {
  static propTypes = {
    accounts: PropTypes.object,
    contact: PropTypes.bool,
    empty: PropTypes.bool
  };

  render () {
    return (
      <div className={ styles.list }>
        { this.renderAccounts() }
      </div>
    );
  }

  renderAccounts () {
    const { accounts, contact, empty } = this.props;

    if (empty) {
      return (
        <Container className={ styles.empty }>
          <div>
            There are currently no accounts or addresses to display.
          </div>
        </Container>
      );
    }

    return Object.keys(accounts).map((address, idx) => {
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
