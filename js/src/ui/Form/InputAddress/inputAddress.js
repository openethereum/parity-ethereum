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

import Input from '../Input';
import IdentityIcon from '../../IdentityIcon';
import util from '../../../api/util';

import styles from './inputAddress.css';

class InputAddress extends Component {
  static propTypes = {
    className: PropTypes.string,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    accountsInfo: PropTypes.object,
    tokens: PropTypes.object,
    text: PropTypes.bool,
    onChange: PropTypes.func,
    onSubmit: PropTypes.func
  };

  render () {
    const { className, disabled, error, label, hint, value, text, onSubmit, accountsInfo, tokens } = this.props;

    const account = accountsInfo[value] || tokens[value];
    const hasAccount = account && (!account.meta || !account.meta.deleted);

    const icon = this.renderIcon();

    const classes = [ className ];
    classes.push(!icon ? styles.inputEmpty : styles.input);

    return (
      <div className={ styles.container }>
        <Input
          className={ classes.join(' ') }
          disabled={ disabled }
          label={ label }
          hint={ hint }
          error={ error }
          value={ text && hasAccount ? account.name : value }
          onChange={ this.handleInputChange }
          onSubmit={ onSubmit }
          allowCopy={ disabled ? value : false }
        />
        { icon }
      </div>
    );
  }

  renderIcon () {
    const { value, disabled } = this.props;

    if (!value || !value.length || !util.isAddressValid(value)) {
      return null;
    }

    return (
      <div className={ disabled ? styles.iconDisabled : styles.icon }>
        <IdentityIcon
          inline center
          address={ value } />
      </div>
    );
  }

  handleInputChange = (event, value) => {
    const isEmpty = (value.length === 0);

    this.setState({ isEmpty });

    if (!/^0x/.test(value) && util.isAddressValid(`0x${value}`)) {
      return this.props.onChange(event, `0x${value}`);
    }

    this.props.onChange(event, value);
  }
}

function mapStateToProps (state) {
  const { accountsInfo } = state.personal;
  const { tokens } = state.balances;

  return {
    accountsInfo,
    tokens
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(InputAddress);
