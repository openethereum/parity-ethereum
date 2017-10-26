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

import React, { Component, PropTypes } from 'react';
import { MenuItem, SelectField } from 'material-ui';

import { nodeOrStringProptype } from '~/util/proptypes';

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
    error: nodeOrStringProptype(),
    hint: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onKeyDown: PropTypes.func,
    type: PropTypes.string,
    value: PropTypes.any,
    values: PropTypes.array
  }

  render () {
    const { className, disabled, error, hint, label, onBlur, onChange, onKeyDown, value } = this.props;

    return (
      <SelectField
        autoComplete='off'
        className={ className }
        disabled={ disabled }
        errorText={ error }
        floatingLabelFixed
        floatingLabelText={ label }
        fullWidth
        hintText={ hint }
        id={ NAME_ID }
        name={ NAME_ID }
        onBlur={ onBlur }
        onChange={ onChange }
        onKeyDown={ onKeyDown }
        underlineDisabledStyle={ UNDERLINE_DISABLED }
        underlineStyle={ UNDERLINE_NORMAL }
        value={ value }
      >
        { this.renderChildren() }
      </SelectField>
    );
  }

  renderChildren () {
    const { children, values } = this.props;

    if (children) {
      return children;
    }

    if (!values) {
      return null;
    }

    return values.map((data, index) => {
      const { name = index, value = index } = data;

      return (
        <MenuItem
          key={ index }
          label={ name }
          value={ value }
        >
          { name }
        </MenuItem>
      );
    });
  }
}
