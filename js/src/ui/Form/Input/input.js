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
import { noop } from 'lodash';
import keycode from 'keycode';
import { Input as SemanticInput } from 'semantic-ui-react';

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';
import { parseI18NString } from '@parity/shared/util/messages';

import CopyToClipboard from '~/ui/CopyToClipboard';
import LabelWrapper from '~/ui/Form/LabelWrapper';

import styles from './input.css';

const NAME_ID = ' ';

export default class Input extends Component {
  static contextTypes = {
    intl: React.PropTypes.object.isRequired
  };

  static propTypes = {
    allowCopy: PropTypes.oneOfType([
      PropTypes.bool,
      PropTypes.string
    ]),
    allowPaste: PropTypes.bool,
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
    focused: PropTypes.bool,
    readOnly: PropTypes.bool,
    hint: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    max: PropTypes.any,
    min: PropTypes.any,
    onBlur: PropTypes.func,
    onChange: PropTypes.func,
    onClick: PropTypes.func,
    onDefaultAction: PropTypes.func,
    onFocus: PropTypes.func,
    onKeyDown: PropTypes.func,
    onSubmit: PropTypes.func,
    submitOnBlur: PropTypes.bool,
    step: PropTypes.number,
    style: PropTypes.object,
    tabIndex: PropTypes.number,
    type: PropTypes.string,
    value: PropTypes.oneOfType([
      PropTypes.number,
      PropTypes.string,
      PropTypes.node
    ])
  };

  static defaultProps = {
    allowCopy: false,
    allowPaste: true,
    escape: 'initial',
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

  // TODO: autoFocus not being used (yet)
  // TODO: multiLine not part of the implementation (need TextArea input)
  render () {
    const { children, className, defaultValue, disabled, error, hint, label, max, min, onClick, readOnly, step, style, tabIndex, type } = this.props;
    const { value } = this.state;

    return (
      <LabelWrapper
        className={ `${styles.container} ${className}` }
        label={ label }
      >
        <SemanticInput
          className={ styles.input }
          disabled={ disabled }
          error={ !!error }
          fluid
          id={ NAME_ID }
          max={ max }
          min={ min }
          name={ NAME_ID }
          onBlur={ this.onBlur }
          onChange={ this.onChange }
          onClick={ onClick }
          onKeyDown={ this.onKeyDown }
          onKeyUp={ this.onKeyUp }
          onFocus={ this.onFocus }
          onPaste={ this.onPaste }
          placeholder={ parseI18NString(this.context, hint) }
          readOnly={ readOnly }
          ref='input'
          step={ step }
          style={ style }
          tabIndex={ tabIndex }
          value={ parseI18NString(this.context, value || defaultValue) }
        >
          { this.renderCopyButton() }
          <input
            type={ type }
          />
          { children }
        </SemanticInput>
      </LabelWrapper>
    );
  }

  renderCopyButton () {
    const { allowCopy } = this.props;
    const { value } = this.state;

    if (!allowCopy) {
      return null;
    }

    const text = typeof allowCopy === 'string'
      ? allowCopy
      : value.toString();

    return (
      <div className={ styles.copy }>
        <CopyToClipboard data={ text } />
      </div>
    );
  }

<<<<<<< HEAD
  onChange = (event, { value }) => {
=======
  onChange = (event, value) => {
    if (!this.props.allowPaste) {
      if (value.length - this.state.value.length > 8) {
        return;
      }
    }

>>>>>>> master
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
