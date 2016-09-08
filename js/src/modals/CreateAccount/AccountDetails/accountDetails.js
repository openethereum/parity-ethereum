import React, { Component, PropTypes } from 'react';

import Form, { Input } from '../../../ui/Form';
import IdentityIcon from '../../../ui/IdentityIcon';

import styles from './accountDetails.css';

export default class AccountDetails extends Component {
  static propTypes = {
    address: PropTypes.string,
    name: PropTypes.string,
    phrase: PropTypes.string
  }

  render () {
    const { address, name } = this.props;

    return (
      <Form>
        <IdentityIcon
          className={ styles.icon }
          address={ address } />
        <div className={ styles.details }>
          <Input
            disabled
            hint='a descriptive name for the account'
            label='account name'
            value={ name } />
          <Input
            disabled
            hint='the network address for the account'
            label='address'
            value={ address } />
          { this.renderPhrase() }
        </div>
      </Form>
    );
  }

  renderPhrase () {
    const { phrase } = this.props;

    if (!phrase) {
      return null;
    }

    return (
      <Input
        disabled
        hint='the account recovery phrase'
        label='account recovery phrase (take note)'
        multiLine
        rows={ 1 }
        value={ phrase } />
    );
  }
}
