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
import { noop } from 'lodash';

import CopyToClipboard from '../../CopyToClipboard';

import styles from './input.css';

// TODO: duplicated in Select
const UNDERLINE_DISABLED = {
  borderBottom: 'dotted 2px',
  borderColor: 'rgba(255, 255, 255, 0.125)' // 'transparent' // 'rgba(255, 255, 255, 0.298039)'
};

const UNDERLINE_READONLY = {
  ...UNDERLINE_DISABLED,
  cursor: 'text'
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
    readOnly: PropTypes.bool,
    allowCopy: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.bool
    ]),
    floatCopy: PropTypes.bool,
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
    hideUnderline: PropTypes.bool,
    value: PropTypes.oneOfType([
      PropTypes.number, PropTypes.string
    ]),
    min: PropTypes.any,
    max: PropTypes.any
  };

  static defaultProps = {
    submitOnBlur: true,
    readOnly: false,
    allowCopy: false,
    hideUnderline: false,
    floatCopy: false
  }

  state = {
    value: this.props.value || ''
  }

  componentWillReceiveProps (newProps) {
    if ((newProps.value !== this.props.value) && (newProps.value !== this.state.value)) {
      this.setValue(newProps.value);
    }
  }

  render () {
    const { value } = this.state;
    const { children, className, hideUnderline, disabled, error, label, hint, multiLine, rows, type, min, max } = this.props;

    const readOnly = this.props.readOnly || disabled;

    const inputStyle = { overflow: 'hidden' };
    const textFieldStyle = {};

    if (readOnly) {
      inputStyle.cursor = 'text';
    }

    if (hideUnderline && !hint) {
      textFieldStyle.height = 'initial';
    }

    return (
      <div className={ styles.container }>
        { this.renderCopyButton() }
        <TextField
          autoComplete='off'
          className={ className }
          style={ textFieldStyle }

          readOnly={ readOnly }

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
          underlineStyle={ readOnly ? UNDERLINE_READONLY : UNDERLINE_NORMAL }
          underlineFocusStyle={ readOnly ? { display: 'none' } : null }
          underlineShow={ !hideUnderline }
          value={ value }
          onBlur={ this.onBlur }
          onChange={ this.onChange }
          onKeyDown={ this.onKeyDown }
          onPaste={ this.onPaste }
          inputStyle={ inputStyle }
          min={ min }
          max={ max }
        >
          { children }
        </TextField>
      </div>
    );
  }

  renderCopyButton () {
    const { allowCopy, label, hint, floatCopy } = this.props;
    const { value } = this.state;

    if (!allowCopy) {
      return null;
    }

    const style = {
      marginBottom: 13
    };

    const text = typeof allowCopy === 'string'
      ? allowCopy
      : value;

    if (!label) {
      style.marginBottom = 2;
    } else if (label && !hint) {
      style.marginBottom = 4;
    } else if (label && hint) {
      style.marginBottom = 10;
    }

    if (floatCopy) {
      style.position = 'absolute';
      style.left = -24;
      style.bottom = style.marginBottom;
      style.marginBottom = 0;
    }

    return (
      <div className={ styles.copy } style={ style }>
        <CopyToClipboard data={ text } />
      </div>
    );
  }

  onChange = (event, value) => {
    event.persist();
    this.setValue(value, () => {
      this.props.onChange && this.props.onChange(event, value);
    });
  }

  onBlur = (event) => {
    const { value } = event.target;
    const { submitOnBlur } = this.props;

    if (submitOnBlur) {
      this.onSubmit(value);
    }

    this.props.onBlur && this.props.onBlur(event);
  }

  onPaste = (event) => {
    const value = event.clipboardData.getData('Text');

    window.setTimeout(() => {
      this.onSubmit(value);
    }, 0);
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
    this.setValue(value, () => {
      this.props.onSubmit && this.props.onSubmit(value);
    });
  }

  setValue (value, cb = noop) {
    this.setState({ value }, cb);
  }
}
