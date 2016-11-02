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

import CopyToClipboard from 'react-copy-to-clipboard';
import CopyIcon from 'material-ui/svg-icons/content/content-copy';
import { TextField, IconButton } from 'material-ui';
import { lightWhite, fullWhite } from 'material-ui/styles/colors';

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
    copiable: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.bool
    ]),
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
  };

  static defaultProps = {
    submitOnBlur: true,
    readOnly: false,
    copiable: false
  }

  state = {
    value: this.props.value || '',
    copied: false
  }

  componentWillReceiveProps (newProps) {
    if (newProps.value !== this.props.value) {
      this.setValue(newProps.value);
    }
  }

  render () {
    const { value } = this.state;
    const { children, className, disabled, error, label, hint, multiLine, rows, type } = this.props;

    const readOnly = this.props.readOnly || disabled;

    return (
      <TextField
        autoComplete='off'
        className={ className }

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
        value={ value }
        onBlur={ this.onBlur }
        onChange={ this.onChange }
        onKeyDown={ this.onKeyDown }
        inputStyle={ readOnly ? { cursor: 'text' } : null }
      >
        { children }
        { this.renderCopyButton() }
      </TextField>
    );
  }

  renderCopyButton () {
    const { copiable } = this.props;
    const { copied, value } = this.state;

    if (!copiable) {
      return null;
    }

    const text = typeof copiable === 'string'
      ? copiable
      : value;

    return (
      <CopyToClipboard
        onCopy={ this.handleCopy }
        text={ text } >
        <IconButton
          tooltip='Copy to clipboard'
          tooltipPosition='top-center'
          style={ {
            width: 32,
            height: 16,
            padding: 0
          } }
          iconStyle={ {
            width: 16,
            height: 16
          } }>
          <CopyIcon
            color={ copied ? lightWhite : fullWhite }
          />
        </IconButton>
      </CopyToClipboard>
    );
  }

  handleCopy = () => {
    this.setState({ copied: true }, () => {
      window.setTimeout(() => {
        this.setState({ copied: false });
      }, 4000);
    });
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
    this.setState({ value });
  }
}
