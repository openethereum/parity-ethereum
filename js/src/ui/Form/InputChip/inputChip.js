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
import { Chip } from 'material-ui';
import ChipInput from 'material-ui-chip-input';
import { blue300 } from 'material-ui/styles/colors';

import styles from './inputChip.css';

export default class InputChip extends Component {
  static propTypes = {
    className: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    value: PropTypes.array.isRequired,
    onRequestAdd: PropTypes.func,
    onRequestDelete: PropTypes.func,
    onChange: PropTypes.func
  }

  render () {
    const { className, hint, label, value, onRequestAdd, onRequestDelete } = this.props;
    const classes = `${className}`;

    return (
      <ChipInput
        className={ classes }
        ref='chipInput'
        value={ value }
        clearOnBlur={ false }
        chipRenderer={ this.chipRenderer }
        floatingLabelText={ label }
        hintText={ hint }

        onRequestAdd={ onRequestAdd }
        onRequestDelete={ onRequestDelete }
        onUpdateInput={ this.onChange }

        floatingLabelFixed
        fullWidth
        hintStyle={ {
          bottom: 17,
          left: 1,
          transition: 'none'
        } }
        inputStyle={ {
          marginBottom: 18
        } }
        textFieldStyle={ {
          height: 42
        } } />
    );
  }

  chipRenderer = (state, key) => {
    const { value, isFocused, isDisabled, handleClick, handleRequestDelete } = state;

    return (
      <Chip
        key={ key }
        className={ styles.chip }
        style={ {
          margin: '8px 8px 0 0',
          float: 'left',
          pointerEvents: isDisabled ? 'none' : undefined,
          alignItems: 'center'
        } }
        labelStyle={ {
          paddingRight: 6,
          fontSize: '0.9rem',
          lineHeight: 'initial'
        } }
        backgroundColor={ isFocused ? blue300 : 'rgba(50, 50, 50, 0.73)' }
        onTouchTap={ handleClick }
        onRequestDelete={ handleRequestDelete }
      >
        { value }
      </Chip>
    );
  }

  onChange = (value) => {
    const { onChange } = this.props;
    const tokens = value.split(/[\s,;]+/);
    const newTokens = tokens
      .slice(0, -1)
      .filter((t) => t.length);

    onChange(newTokens);

    const inputValue = tokens.slice(-1)[0].trim();
    this.refs.chipInput.setState({ inputValue });
  }
}
