// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import AddressSelect from '../AddressSelect';

class InputAddressSelect extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired,
    contracts: PropTypes.object.isRequired,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    onChange: PropTypes.func
  };

  render () {
    const { accounts, contacts, contracts, label, hint, error, value, onChange } = this.props;

    return (
      <AddressSelect
        allowInput
        accounts={ accounts }
        contacts={ contacts }
        contracts={ contracts }
        error={ error }
        label={ label }
        hint={ hint }
        value={ value }
        onChange={ onChange } />
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

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(InputAddressSelect);
