import React, { Component, PropTypes } from 'react';

import { Actionbar, Page } from '../../ui';

import Header from '../Account/Header';
import Transactions from '../Account/Transactions';

import styles from './address.css';

export default class Address extends Component {
  static contextTypes = {
    contacts: PropTypes.array
  }

  static propTypes = {
    params: PropTypes.object
  }

  render () {
    const { contacts } = this.context;
    const { address } = this.props.params;
    const contact = contacts.find((_account) => _account.address === address);

    if (!contact) {
      return null;
    }

    return (
      <div className={ styles.address }>
        { this.renderActionbar() }
        <Page>
          <Header
            account={ contact } />
          <Transactions
            address={ address } />
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
    ];

    return (
      <Actionbar
        title='Address Information'
        buttons={ buttons } />
    );
  }
}
