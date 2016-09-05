import React, { Component, PropTypes } from 'react';

import Form, { FormWrap, Input } from '../../../ui/Form';
import IdentityIcon from '../../../ui/IdentityIcon';

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
          padded
          address={ address } />
        <FormWrap>
          <Input
            disabled
            hint='a descriptive name for the account'
            label='account name'
            value={ name } />
        </FormWrap>
        <FormWrap>
          <Input
            disabled
            hint='the network address for the account'
            label='address'
            value={ address } />
        </FormWrap>
        { this.renderPhrase() }
      </Form>
    );
  }

  renderPhrase () {
    const { phrase } = this.props;

    if (!phrase) {
      return null;
    }

    return (
      <FormWrap>
        <Input
          disabled
          hint='the account recovery phrase'
          label='account recovery phrase (take note)'
          multiLine
          rows={ 1 }
          value={ phrase } />
      </FormWrap>
    );
  }
}
