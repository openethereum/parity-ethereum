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
import { uniq } from 'lodash';

import styles from './inputChip.css';

export default class InputChip extends Component {
  static propTypes = {
    tokens: PropTypes.array.isRequired,
    className: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    onTokensChange: PropTypes.func,
    onInputChange: PropTypes.func,
    onBlur: PropTypes.func,
    clearOnBlur: PropTypes.bool
  }

  static defaultProps = {
    clearOnBlur: false
  }

  render () {
    const { clearOnBlur, className, hint, label, tokens } = this.props;
    const classes = `${className}`;

    return (
      <ChipInput
        className={ classes }
        ref='chipInput'

        value={ tokens }
        clearOnBlur={ clearOnBlur }
        floatingLabelText={ label }
        hintText={ hint }

        chipRenderer={ this.chipRenderer }

        onBlur={ this.handleBlur }
        onRequestAdd={ this.handleTokenAdd }
        onRequestDelete={ this.handleTokenDelete }
        onUpdateInput={ this.handleInputChange }

        floatingLabelFixed
        fullWidth

        hintStyle={ {
          bottom: 16,
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

  handleBlur = () => {
    const { onBlur } = this.props;

    if (typeof onBlur === 'function') {
      onBlur();
    }
  }

  handleTokenAdd = (value) => {
    const { tokens, onInputChange } = this.props;

    const newTokens = uniq([].concat(tokens, value));

    this.handleTokensChange(newTokens);

    if (value === this.refs.chipInput.state.inputValue && typeof onInputChange === 'function') {
      onInputChange('');
    }
  }

  handleTokenDelete = (value) => {
    const { tokens } = this.props;

    const newTokens = uniq([]
      .concat(tokens)
      .filter(v => v !== value));

    this.handleTokensChange(newTokens);
    this.refs.chipInput.focus();
  }

  handleInputChange = (value) => {
    const { onInputChange } = this.props;

    const splitTokens = value.split(/[\s,;]/);

    const inputValue = (splitTokens.length <= 1)
      ? value
      : splitTokens.slice(-1)[0].trim();

    this.refs.chipInput.setState({ inputValue });

    if (splitTokens.length > 1) {
      const tokensToAdd = splitTokens.slice(0, -1);
      tokensToAdd.forEach(token => this.handleTokenAdd(token));
    }

    if (typeof onInputChange === 'function') {
      onInputChange(inputValue);
    }
  }

  handleTokensChange = (tokens) => {
    const { onTokensChange } = this.props;

    onTokensChange(tokens.filter(token => token && token.length > 0));
  }

}
