import React, { Component, PropTypes } from 'react';

import Form, { FormWrap, Input } from '../../../Form';
import IdentityIcon from '../../../IdentityIcon';

export default class RecoverAccount extends Component {
  static propTypes = {
    accountAddress: PropTypes.string,
    accountName: PropTypes.string,
    accountPhrase: PropTypes.string,
    visible: PropTypes.bool.isRequired
  }

  render () {
    if (!this.props.visible) {
      return null;
    }

    return (
      <Form>
        <IdentityIcon
          address={ this.props.accountAddress } />
        <FormWrap>
          <Input
            disabled
            hint='a descriptive name for the account'
            label='account name'
            value={ this.props.accountName } />
        </FormWrap>
        <FormWrap>
          <Input
            disabled
            hint='the network address for the account'
            label='address'
            value={ this.props.accountAddress } />
        </FormWrap>
        <FormWrap>
          <Input
            disabled
            hint='the account recovery phrase'
            label='recovery phrase'
            multiLine
            rows={ 2 }
            value={ this.props.accountPhrase } />
        </FormWrap>
      </Form>
    );
  }
}
