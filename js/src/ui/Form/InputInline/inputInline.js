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

import { nodeOrStringProptype } from '~/util/proptypes';

import Input from '../Input';

import styles from './inputInline.css';

export default class InputInline extends Component {
  static propTypes = {
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onSubmit: PropTypes.func,
    onKeyDown: PropTypes.func,
    type: PropTypes.string,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ]),
    static: nodeOrStringProptype()
  }

  state = {
    editing: false
  }

  render () {
    const { editing } = this.state;
    const { error, label, hint, type, value } = this.props;

    if (!editing) {
      return (
        <div
          className={ styles.inlineedit }
          onClick={ this.onToggle }
        >
          { this.props.static || value }
        </div>
      );
    }

    return (
      <Input
        error={ error }
        label={ label }
        hint={ hint }
        type={ type }
        value={ value }
        onBlur={ this.onBlur }
        onChange={ this.props.onChange }
        onKeyDown={ this.onKeyDown }
        onSubmit={ this.props.onSubmit }
      />
    );
  }

  onBlur = () => {
    this.onToggle();

    if (this.props.onBlur) {
      this.props.onBlur();
    }
  }

  onToggle = () => {
    this.setState({
      editing: !this.state.editing
    });
  }

  onKeyDown = (event) => {
    if (event.keyCode === 13) {
      this.onToggle();
    }

    this.props.onKeyDown && this.props.onKeyDown(event);
  }
}
