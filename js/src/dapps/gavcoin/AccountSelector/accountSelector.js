import React, { Component, PropTypes } from 'react';

import { SelectField } from 'material-ui';

import { renderAccounts } from './render';

const NAME_ID = ' ';
let lastSelectedAccount = {};

export default class AccountSelect extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    account: PropTypes.object,
    gavBalance: PropTypes.bool,
    onSelect: PropTypes.func,
    errorText: PropTypes.string,
    floatingLabelText: PropTypes.string,
    hintText: PropTypes.string
  }

  componentDidMount () {
    this.props.onSelect(lastSelectedAccount);
  }

  render () {
    const { account, accounts, errorText, floatingLabelText, gavBalance, hintText } = this.props;

    return (
      <SelectField
        autoComplete='off'
        floatingLabelFixed
        floatingLabelText={ floatingLabelText }
        fullWidth
        hintText={ hintText }
        errorText={ errorText }
        name={ NAME_ID }
        id={ NAME_ID }
        value={ account }
        onChange={ this.onSelect }>
        { renderAccounts(accounts, { gavBalance }) }
      </SelectField>
    );
  }

  onSelect = (event, idx, account) => {
    lastSelectedAccount = account || {};
    this.props.onSelect(lastSelectedAccount);
  }
}
