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

import { DatePicker } from 'material-ui';
import React, { Component, PropTypes } from 'react';

import Label from '../Label';

import styles from './inputDate.css';

// NOTE: Has to be larger than Signer overlay Z, aligns with ../InputTime
const DIALOG_STYLE = { zIndex: 10010 };

export default class InputDate extends Component {
  static propTypes = {
    className: PropTypes.string,
    hint: PropTypes.node,
    label: PropTypes.node,
    onChange: PropTypes.func,
    value: PropTypes.object.isRequired
  };

  render () {
    const { className, hint, label, onChange, value } = this.props;

    return (
      <div className={ [styles.container, className].join(' ') }>
        <Label label={ label } />
        <DatePicker
          autoOk
          className={ styles.input }
          dialogContainerStyle={ DIALOG_STYLE }
          hintText={ hint }
          onChange={ onChange }
          value={ value }
        />
      </div>
    );
  }
}
