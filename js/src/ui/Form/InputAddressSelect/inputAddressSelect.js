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
import normalize from 'normalize-for-search';

import IdentityIcon from '../../IdentityIcon';
import { validateAddress } from '../../../util/validation';

import styles from './inputAddressSelect.css';

const computeHaystack = (accounts, contacts) => {
  const data = Object.assign({}, contacts, accounts);
  return Object.values(data)
  .map((value) => Object.assign(
    Object.create(value),
    { tokens: normalize(value.name) }
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
    entries: [],
    address: ''
  }

  componentWillReceiveProps (nextProps) {
    const { accounts, contacts } = nextProps;
    // TODO diff against last props

    this.setState({
      haystack: computeHaystack(accounts, contacts)
    });
  }

  render () {
    const { label, hint, error } = this.props;
    const { entries, address } = this.state;

    const choices = entries.map((data) => ({
      value: this.renderAddress(data), text: data.name
    }));

    return (
      <div className={ styles.wrapper }>
        <IdentityIcon
          className={ styles.icon }
          address={ address }
          inline
        />
        <AutoComplete
          floatingLabelText={ label }
          hintText={ hint }
          errorText={ error }
          dataSource={ choices }
          onNewRequest={ this.onNewRequest }
          onUpdateInput={ this.onUpdateInput }
          fullWidth={ true }
        />
      </div>
    );
  }

  renderAddress = (data) => {
    const icon = ( <IdentityIcon address={ data.address } inline /> );
    // TODO move those styles down there to a better place
    return (
      <MenuItem
        primaryText={ data.name }
        key={ data.address }
        leftIcon={ icon }
        innerDivStyle={ { display: 'flex', paddingLeft: '1em', paddingRight: '1em', alignItems: 'center' } }
      />
    );
  }

  onNewRequest = (data) => {
    this.setState({
      address: data.value.key
    });
    this.props.onChange(null, data.value.key);
  };

  onUpdateInput = (value) => {
    value = value.trim()
    if (value === '') {
      this.setState({ entries: [] });
      return;
    }

    const needle = normalize(value);
    const entries = this.state.haystack
      .filter((data) => data.tokens.indexOf(needle) >= 0);

    this.setState({
      entries,
      address: value
    });

    const isValid = !validateAddress(value).addressError;
    if (isValid || entries.length === 0) {
      this.props.onChange(null, value);
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
