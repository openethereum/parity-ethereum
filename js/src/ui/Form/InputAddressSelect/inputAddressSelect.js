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
import { Toggle } from 'material-ui';

import AddressSelect from '../AddressSelect';
import InputAddress from '../InputAddress';

import styles from './inputAddressSelect.css';

class InputAddressSelect extends Component {
  static propTypes = {
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    disabled: PropTypes.bool,
    editing: PropTypes.bool,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    tokens: PropTypes.object,
    onChange: PropTypes.func
  };

  state = {
    editing: this.props.editing || false,
    entries: []
  }

  render () {
    const { editing } = this.state;

    return (
      <div className={ styles.inputselect }>
        { editing ? this.renderInput() : this.renderSelect() }
        <Toggle
          className={ styles.toggle }
          label='Edit'
          labelPosition='right'
          toggled={ editing }
          onToggle={ this.onToggle } />
      </div>
    );
  }

  renderInput () {
    const { disabled, error, hint, label, value, tokens } = this.props;

    return (
      <InputAddress
        disabled={ disabled }
        error={ error }
        hint={ hint }
        label={ label }
        value={ value }
        tokens={ tokens }
        onChange={ this.onChangeInput } />
    );
  }

  renderSelect () {
    const { accounts, contacts, disabled, error, hint, label, value, tokens } = this.props;

    return (
      <AddressSelect
        accounts={ accounts }
        contacts={ contacts }
        disabled={ disabled }
        label={ label }
        hint={ hint }
        error={ error }
        value={ value }
        tokens={ tokens }
        onChange={ this.onChangeSelect } />
    );
  }

  onToggle = () => {
    const { editing } = this.state;

    this.setState({
      editing: !editing
    });
  }

  onChangeInput = (event, value) => {
    this.props.onChange(event, value);
  }

  onChangeSelect = (event, value) => {
    this.props.onChange(event, value);
  }
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
