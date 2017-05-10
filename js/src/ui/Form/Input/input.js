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
import { Input as SemanticInput } from 'semantic-ui-react';
import { noop } from 'lodash';
import keycode from 'keycode';

import { nodeOrStringProptype } from '~/util/proptypes';

import CopyToClipboard from '../../CopyToClipboard';

import styles from './input.css';

export default class Input extends Component {
  static contextTypes = {
    intl: React.PropTypes.object.isRequired
  };

  static propTypes = {
    allowCopy: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.bool
    ]),
    autoFocus: PropTypes.bool,
    children: PropTypes.node,
    className: PropTypes.string,
    defaultValue: PropTypes.string,
    disabled: PropTypes.bool,
    error: nodeOrStringProptype(),
    escape: PropTypes.oneOf([
      'default',
      'initial'
    ]),
    fullWidth: PropTypes.bool,
    fluid: PropTypes.bool,
    focused: PropTypes.bool,
    readOnly: PropTypes.bool,
    hint: nodeOrStringProptype(),
    hideUnderline: PropTypes.bool,
    label: nodeOrStringProptype(),
    max: PropTypes.any,
    min: PropTypes.any,
    multiLine: PropTypes.bool,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onClick: PropTypes.func,
    onDefaultAction: PropTypes.func,
    onFocus: PropTypes.func,
    onKeyDown: PropTypes.func,
    onSubmit: PropTypes.func,
    rows: PropTypes.number,
    tabIndex: PropTypes.number,
    type: PropTypes.string,
    submitOnBlur: PropTypes.bool,
    step: PropTypes.number,
    style: PropTypes.object,
    value: PropTypes.oneOfType([
      PropTypes.number,
      PropTypes.string,
      PropTypes.node
    ])
  };

  static defaultProps = {
    allowCopy: false,
    escape: 'initial',
    fluid: true,
    hideUnderline: false,
    onBlur: noop,
    onFocus: noop,
    onChange: noop,
    readOnly: false,
    submitOnBlur: true,
    style: {},
    type: 'text'
  }

  state = {
    value: typeof this.props.value === 'undefined'
      ? ''
      : this.props.value
  }

  componentDidMount () {
    const { autoFocus } = this.props;

    if (autoFocus) {
      this.keyInput.focus();
    }
  }

  componentWillReceiveProps (newProps) {
    if ((newProps.value !== this.props.value) && (newProps.value !== this.state.value)) {
      this.setValue(newProps.value);
    }

    if (newProps.focused && !this.props.focused) {
      this.refs.input.setState({ isFocused: true });
    }

    if (!newProps.focused && this.props.focused) {
      this.refs.input.setState({ isFocused: false });
    }
  }

  render () {
    const { value } = this.state;
    const {
      children,
      className,
      defaultValue,
      disabled,
      error,
      fluid,
      focused,
      label,
      onClick,
      style,
      tabIndex,
      type
    } = this.props;

    return (
      <div className={ styles.container } style={ style }>
        { this.renderCopyButton() }
        <SemanticInput
          className={ className }
          defaultValue={ defaultValue }
          disabled={ disabled }
          error={ error }
          fluid={ fullWidth | fluid }
          focus={ focused }
          input={ value }
          label={ label }
          onBlur={ this.onBlur }
          onChange={ this.onChange }
          onClick={ onClick }
          onKeyDown={ this.onKeyDown }
          onKeyUp={ this.onKeyUp }
          onFocus={ this.onFocus }
          onPaste={ this.onPaste }
          placeholder={ hint }
          ref={ this.keyInput }
          style={ style }
          tabIndex={ tabIndex }
          type={ type }
        >
          { children }
          <input />
        </SemanticInput>
      </div>
    );
  }

  renderCopyButton () {
    const { allowCopy, hideUnderline } = this.props;
    const { value } = this.state;

    if (!allowCopy) {
      return null;
    }

    const text = typeof allowCopy === 'string'
      ? allowCopy
      : value.toString();

    const style = hideUnderline
      ? {}
      : { position: 'relative', top: '2px' };

    return (
      <div className={ styles.copy } style={ style }>
        <CopyToClipboard data={ text } />
      </div>
    );
  }

  onChange = (event, value) => {
    event.persist();

    this.setValue(value, () => {
      this.props.onChange(event, value);

      if (this.pasted) {
        this.pasted = false;
        return this.onSubmit(value);
      }
    });
  }

  onBlur = (event) => {
    const { value } = event.target;
    const { submitOnBlur } = this.props;

    if (submitOnBlur) {
      this.onSubmit(value);
    }

    this.props.onBlur(event);
  }

  onFocus = (event) => {
    const { onFocus } = this.props;

    this.intialValue = event.target.value;
    return onFocus(event);
  }

  onPaste = (event) => {
    this.pasted = true;
  }

  onKeyDown = (event) => {
    const codeName = keycode(event);
    const { value } = event.target;

    if (codeName === 'enter') {
      this.onSubmit(value, true);
    }

    this.props.onKeyDown && this.props.onKeyDown(event);
  }

  /**
   * Revert to initial value if pressed ESC key
   * once. Don't do anything (propagate the event) if
   * ESC has been pressed twice in a row (eg. input in a Portal modal).
   *
   * NB: it has to be `onKeyUp` since the Portal is using
   * the `onKeyUp` event to close the modal ; it mustn't be propagated
   * if we only want to revert to initial value
   */
  onKeyUp = (event) => {
    const { escape } = this.props;
    const codeName = keycode(event);

    if (codeName === 'esc' && !this.pressedEsc) {
      event.stopPropagation();
      event.preventDefault();

      this.pressedEsc = true;

      if (escape === 'initial' && this.intialValue !== undefined) {
        return this.onChange(event, this.intialValue);
      }

      if (escape === 'default' && this.props.defaultValue !== undefined) {
        return this.onSubmit(this.props.defaultValue);
      }
    } else if (this.pressedEsc) {
      this.pressedEsc = false;
    }
  }

  onSubmit = (value, performDefault) => {
    const { onDefaultAction, onSubmit } = this.props;

    this.setValue(value, () => {
      if (onSubmit) {
        onSubmit(value);
      }

      if (performDefault && onDefaultAction) {
        onDefaultAction();
      }
    });
  }

  setValue (value, cb = noop) {
    this.setState({ value }, cb);
  }
}
