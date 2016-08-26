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
    return (
      <SelectField
        autoComplete='off'
        floatingLabelFixed
        floatingLabelText={ this.props.floatingLabelText }
        fullWidth
        hintText={ this.props.hintText }
        errorText={ this.props.errorText }
        name={ NAME_ID }
        id={ NAME_ID }
        value={ this.props.account }
        onChange={ this.onSelect }>
        { renderAccounts(this.props.accounts, this.props.gavBalance) }
      </SelectField>
    );
  }

  onSelect = (event, idx, account) => {
    lastSelectedAccount = account || {};
    this.props.onSelect(lastSelectedAccount);
  }
}
