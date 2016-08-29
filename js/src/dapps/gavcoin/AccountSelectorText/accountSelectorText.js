import React, { Component, PropTypes } from 'react';
import { TextField } from 'material-ui';

import AccountSelector from '../AccountSelector';
import AccountItem from '../AccountSelector/AccountItem';

const NAME_ID = ' ';

export default class AccountSelectorText extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    account: PropTypes.object,
    errorText: PropTypes.string,
    gavBalance: PropTypes.bool,
    floatingLabelText: PropTypes.string,
    hintText: PropTypes.string,
    selector: PropTypes.bool,
    onChange: PropTypes.func
  }

  render () {
    const { selector } = this.props;

    return selector
      ? this.renderDropDown()
      : this.renderTextField();
  }

  renderDropDown () {
    const { account, accounts, errorText, gavBalance, hintText, floatingLabelText, onChange } = this.props;

    return (
      <AccountSelector
        gavBalance={ gavBalance }
        accounts={ accounts }
        account={ account }
        errorText={ errorText }
        floatingLabelText={ floatingLabelText }
        hintText={ hintText }
        onSelect={ onChange } />
    );
  }

  renderTextField () {
    const { account, errorText, gavBalance, hintText, floatingLabelText } = this.props;

    return (
      <TextField
        autoComplete='off'
        floatingLabelFixed
        floatingLabelText={ floatingLabelText }
        fullWidth
        hintText={ hintText }
        errorText={ errorText }
        name={ NAME_ID }
        id={ NAME_ID }
        value={ account.address || '' }
        onChange={ this.onChangeAddress }>
        <AccountItem
          account={ account }
          gavBalance={ gavBalance } />
      </TextField>
    );
  }

  onChangeAddress = (event, address) => {
    const lower = address.toLowerCase();
    const account = this.props.accounts.find((_account) => _account.address.toLowerCase() === lower);

    this.props.onChange(account || { address });
  }
}
