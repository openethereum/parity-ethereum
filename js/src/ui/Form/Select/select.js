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
import { SelectField } from 'material-ui';

// TODO: duplicated in Input
const UNDERLINE_DISABLED = {
  borderColor: 'rgba(255, 255, 255, 0.298039)' // 'transparent' // 'rgba(255, 255, 255, 0.298039)'
};

const UNDERLINE_NORMAL = {
  borderBottom: 'solid 2px'
};

const NAME_ID = ' ';

export default class Select extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onKeyDown: PropTypes.func,
    type: PropTypes.string,
    value: PropTypes.any
  }

  render () {
    const { disabled, error, label, hint, value, children, className, onBlur, onChange, onKeyDown } = this.props;

    return (
      <SelectField
        className={ className }
        autoComplete='off'
        disabled={ disabled }
        errorText={ error }
        floatingLabelFixed
        floatingLabelText={ label }
        fullWidth
        hintText={ hint }
        name={ NAME_ID }
        id={ NAME_ID }
        underlineDisabledStyle={ UNDERLINE_DISABLED }
        underlineStyle={ UNDERLINE_NORMAL }
        value={ value }
        onBlur={ onBlur }
        onChange={ onChange }
        onKeyDown={ onKeyDown }>
        { children }
      </SelectField>
    );
  }
}
