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

import AddressSelect from '../addressSelect';

class InputAddressSelect extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    onChange: PropTypes.func
  };

  state = {
    address: ''
  }

  render () {
    const { accounts, contacts, label, hint, error, value } = this.props;

    return (
      <AddressSelect
        accounts={ accounts }
        contacts={ contacts }
        error={ error }
        label={ label }
        hint={ hint }
        value={ value }
        onChange={ this.onChange }
        onUpdateInput={ this.onUpdateInput } />
    );
  }

  onChange = (event, address) => {
    const { onChange } = this.props;

    console.log('onChange', event, address);
    onChange(null, address);
  };

  onUpdateInput = (query, choices) => {
    const { api } = this.context;
    const { onChange } = this.props;

    console.log('onUpdateInput', query);

    query = query.trim();
    this.setState({ address: query });

    if (query.slice(0, 2) !== '0x' && api.util.isAddressValid(`0x${query}`)) {
      onChange(null, `0x${query}`);
    } else {
      onChange(null, query);
    }
  };
}

function mapStateToProps (state) {
  const { accounts, contacts } = state.personal;

  return {
    accounts,
    contacts
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(InputAddressSelect);
