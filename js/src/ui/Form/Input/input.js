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

import { TextField } from 'material-ui';

// TODO: duplicated in Select
const UNDERLINE_DISABLED = {
  borderColor: 'rgba(255, 255, 255, 0.298039)' // 'transparent' // 'rgba(255, 255, 255, 0.298039)'
};

const UNDERLINE_NORMAL = {
  borderBottom: 'solid 2px'
};

const NAME_ID = ' ';

export default class Input extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    multiLine: PropTypes.bool,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onKeyDown: PropTypes.func,
    onSubmit: PropTypes.func,
    rows: PropTypes.number,
    type: PropTypes.string,
    submitOnBlur: PropTypes.bool,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ])
  }

  static defaultProps = {
    submitOnBlur: true
  }

  state = {
    value: this.props.value || ''
  }

  componentWillReceiveProps (newProps) {
    if (newProps.value !== this.props.value) {
      this.setValue(newProps.value);
    }
  }

  render () {
    const { value } = this.state;
    const { children, className, disabled, error, label, hint, multiLine, rows, type } = this.props;

    return (
      <TextField
        autoComplete='off'
        className={ className }
        disabled={ disabled }
        errorText={ error }
        floatingLabelFixed
        floatingLabelText={ label }
        fullWidth
        hintText={ hint }
        multiLine={ multiLine }
        name={ NAME_ID }
        id={ NAME_ID }
        rows={ rows }
        type={ type || 'text' }
        underlineDisabledStyle={ UNDERLINE_DISABLED }
        underlineStyle={ UNDERLINE_NORMAL }
        value={ value }
        onBlur={ this.onBlur }
        onChange={ this.onChange }
        onKeyDown={ this.onKeyDown }>
        { children }
      </TextField>
    );
  }

  onChange = (event, value) => {
    this.setValue(value);

    this.props.onChange && this.props.onChange(event, value);
  }

  onBlur = (event) => {
    const { value } = event.target;
    const { submitOnBlur } = this.props;

    if (submitOnBlur) {
      this.onSubmit(value);
    }

    this.props.onBlur && this.props.onBlur(event);
  }

  onKeyDown = (event) => {
    const { value } = event.target;

    if (event.which === 13) {
      this.onSubmit(value);
    } else if (event.which === 27) {
      // TODO ESC, revert to original
    }

    this.props.onKeyDown && this.props.onKeyDown(event);
  }

  onSubmit = (value) => {
    this.setValue(value);

    this.props.onSubmit && this.props.onSubmit(value);
  }

  setValue (value) {
    this.setState({
      value
    });
  }
}
