import React, { Component, PropTypes } from 'react';

import List from '../Accounts/List';
import { Actionbar } from '../../ui';

import styles from './addresses.css';

export default class Addresses extends Component {
  static contextTypes = {
    api: PropTypes.object,
    contacts: PropTypes.array
  }

  render () {
    const { contacts } = this.context;

    return (
      <div className={ styles.addresses }>
        { this.renderActionbar() }
        <List accounts={ contacts } />
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
}
