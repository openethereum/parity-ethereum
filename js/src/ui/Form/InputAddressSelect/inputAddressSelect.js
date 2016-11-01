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

import AutoComplete from 'material-ui/AutoComplete';
import MenuItem from 'material-ui/MenuItem';

import IdentityIcon from '../../IdentityIcon';

import normalize from 'normalize-for-search';

// import styles from './inputAddressSelect.css';

const computeHaystack = (accounts, contacts) => {
  const data = Object.assign({}, contacts, accounts);
  return Object.values(data)
  .map((value) => Object.assign(
    Object.create(value),
    { tokens: normalize(value.name).trim() }
  ));
};

class InputAddressSelect extends Component {
  static propTypes = {
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    tokens: PropTypes.object,
    onChange: PropTypes.func
  };

  static defaultProps = {
    onChange: () => {}
  };

  state = {
    haystack: [],
    entries: []
  }

  componentWillReceiveProps (nextProps) {
    const { accounts, contacts } = nextProps;
    // TODO diff against last props

    this.setState({
      haystack: computeHaystack(accounts, contacts)
    });
  }

  render () {
    const { label, hint } = this.props;
    const { entries } = this.state;

    const choices = entries.map((data) => ({
      // text: this.renderAddress(data), value: data.address
      text: data.name, value: data.address
    }));

    return (
      <AutoComplete
        floatingLabelText={ label }
        hintText={ hint }
        dataSource={ choices }
        onUpdateInput={ this.onUpdateInput }
        fullWidth={ true }
      />
    );
  }

  renderAddress = (data) => {
    const icon = ( <IdentityIcon address={ data.address } inline center /> );
    return (
      <MenuItem
        primaryText={ data.name }
        key={ data.address }
        leftIcon={ icon }
      />
    );
  }

  onUpdateInput = (value) => {
    if (value.trim() === '') {
      this.setState({ entries: [] });
      return;
    }

    const { haystack } = this.state;
    const needle = normalize(value).trim();

    const entries = haystack.filter((data) => data.tokens.indexOf(needle) >= 0);

    this.setState({
      entries
    });
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
