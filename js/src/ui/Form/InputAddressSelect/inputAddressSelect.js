// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { pick } from 'lodash';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import { nodeOrStringProptype } from '~/util/proptypes';

import AddressSelect from '../AddressSelect';

class InputAddressSelect extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired,
    contracts: PropTypes.object.isRequired,

    allowCopy: PropTypes.bool,
    allowedValues: PropTypes.array,
    className: PropTypes.string,
    error: nodeOrStringProptype(),
    hint: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    onChange: PropTypes.func,
    readOnly: PropTypes.bool,
    value: PropTypes.string
  };

  render () {
    const { accounts, allowCopy, allowedValues, className, contacts, contracts, label, hint, error, value, onChange, readOnly } = this.props;
    // Add the currently selected value to the list
    // of allowed values, if any given
    const nextAllowedValues = allowedValues
      ? [].concat(allowedValues, value || [])
      : null;

    const filteredAccounts = nextAllowedValues
      ? pick(accounts, nextAllowedValues)
      : accounts;

    const filteredContacts = nextAllowedValues
      ? pick(contacts, nextAllowedValues)
      : accounts;

    const filteredContracts = nextAllowedValues
      ? pick(contracts, nextAllowedValues)
      : accounts;

    return (
      <AddressSelect
        allowCopy={ allowCopy }
        allowInput
        accounts={ filteredAccounts }
        className={ className }
        contacts={ filteredContacts }
        contracts={ filteredContracts }
        error={ error }
        hint={ hint }
        label={ label }
        onChange={ onChange }
        readOnly={ readOnly }
        value={ value }
      />
    );
  }
}

function mapStateToProps (state) {
  const { accounts, contacts, contracts } = state.personal;

  return {
    accounts,
    contacts,
    contracts
  };
}

export default connect(
  mapStateToProps,
  null
)(InputAddressSelect);
