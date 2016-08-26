import React, { Component, PropTypes } from 'react';

import { TextField } from 'material-ui';

const NAME_ID = ' ';

export default class AccountTextField extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    account: PropTypes.object,
    errorText: PropTypes.string,
    floatingLabelText: PropTypes.string,
    hintText: PropTypes.string,
    onChange: PropTypes.func
  }

  render () {
    return (
      <TextField
        autoComplete='off'
        floatingLabelFixed
        floatingLabelText={ this.props.floatingLabelText }
        fullWidth
        hintText={ this.props.hintText }
        errorText={ this.props.errorText }
        name={ NAME_ID }
        id={ NAME_ID }
        value={ this.props.account.address || '' }
        onChange={ this.onChangeAddress } />
    );
  }

  onChangeAddress = (event, address) => {
    const lower = address.toLowerCase();
    const account = this.props.accounts.find((_account) => _account.address.toLowerCase() === lower);

    this.props.onChange(account || { address });
  }
}
