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

import Input from '../Input';
import IdentityIcon from '../../IdentityIcon';

import styles from './inputAddress.css';

export default class InputAddress extends Component {
  static propTypes = {
    className: PropTypes.string,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    nameValue: PropTypes.string,
    tokens: PropTypes.object,
    onChange: PropTypes.func,
    onSubmit: PropTypes.func
  };

  render () {
    const { className, disabled, error, label, hint, value, nameValue, onChange, onSubmit } = this.props;
    const classes = `${styles.input} ${className}`;

    return (
      <div className={ styles.container }>
        <Input
          className={ classes }
          disabled={ disabled }
          label={ label }
          hint={ hint }
          error={ error }
          value={ nameValue || value }
          onChange={ onChange }
          onSubmit={ onSubmit } />
        { this.renderIcon() }
      </div>
    );
  }

  renderIcon () {
    const { value, tokens } = this.props;

    if (!value || !value.length) {
      return null;
    }

    return (
      <div className={ styles.icon }>
        <IdentityIcon
          inline center
          tokens={ tokens }
          address={ value } />
      </div>
    );
  }
}
