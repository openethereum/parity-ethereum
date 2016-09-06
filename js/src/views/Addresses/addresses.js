import React, { Component, PropTypes } from 'react';

import Summary from '../Accounts/Summary';
import { Actionbar, Container } from '../../ui';

import styles from './addresses.css';

export default class Addresses extends Component {
  static contextTypes = {
    api: PropTypes.object,
    contacts: PropTypes.array
  }

  render () {
    console.log(this.renderAddresses());

    return (
      <div>
        { this.renderActionbar() }
        <div className={ styles.addresses }>
          { this.renderAddresses() }
        </div>
      </div>
    );
  }

  renderActionbar () {
    return (
      <Actionbar
        className={ styles.toolbar }
        title='Saved Addresses' />
    );
  }

  renderAddresses () {
    const { contacts } = this.context;

    if (!contacts || !contacts.length) {
      return (
        <Container className={ styles.empty }>
          There are currently no saved addresses.
        </Container>
      );
    }

    return contacts.map((contact, idx) => {
      return (
        <div
          className={ styles.address }
          key={ contact.address }>
          <Summary
            contact
            account={ contact } />
        </div>
      );
    });
  }
}
