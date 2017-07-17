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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import keycode from 'keycode';
import { uniq } from 'lodash';
import { Input as SemanticInput } from 'semantic-ui-react';

import { parseI18NString } from '@parity/shared/util/messages';
import { arrayOrObjectProptype, nodeOrStringProptype } from '@parity/shared/util/proptypes';

import LabelWrapper from '~/ui/Form/LabelWrapper';

import Chip from './Chip';

export default class InputChip extends Component {
  static contextTypes = {
    intl: PropTypes.object
  };

  static propTypes = {
    autoFocus: PropTypes.bool,
    addOnBlur: PropTypes.bool,
    clearOnBlur: PropTypes.bool,
    className: PropTypes.string,
    hint: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    onTokensChange: PropTypes.func,
    onBlur: PropTypes.func,
    tokens: arrayOrObjectProptype().isRequired
  };

  static defaultProps = {
    clearOnBlur: false,
    addOnBlur: false
  };

  state = {
    textValue: ''
  };

  // TODO: autoFocus to be implemented (same as with Form/Input)
  render () {
    const { className, hint, label, tokens } = this.props;
    const { textValue } = this.state;

    return (
      <LabelWrapper
        className={ className }
        label={ label }
      >
        <SemanticInput
          fluid
          onBlur={ this.onBlur }
          onChange={ this.onChange }
          onKeyDown={ this.onKeyDown }
          placeholder={ parseI18NString(this.context, hint) }
          ref='chipInput'
          value={ textValue }
        >
          <input />
        </SemanticInput>
        <div>
          { tokens.map(this.renderChip) }
        </div>
      </LabelWrapper>
    );
  }

  renderChip = (chip) => {
    const onDelete = (event) => this.handleTokenDelete(chip);

    return (
      <Chip
        key={ chip }
        label={ chip }
        onDelete={ onDelete }
      />
    );
  }

  handleTokenAdd = (value) => {
    const { tokens } = this.props;
    const newTokens = uniq([].concat(tokens, value));

    this.handleTokensChange(newTokens);
    this.setState({ textValue: '' });
  }

  handleTokenDelete = (value) => {
    const { tokens } = this.props;

    this.handleTokensChange(
      uniq(
        []
          .concat(tokens)
          .filter((token) => token !== value)
      )
    );

    this.refs.chipInput.focus();
  }

  handleTokensChange = (tokens) => {
    const { onTokensChange } = this.props;

    onTokensChange(tokens.filter((token) => token && token.length > 0));
  }

  onBlur = () => {
    const { onBlur, addOnBlur } = this.props;

    if (addOnBlur) {
      const { textValue } = this.state;

      this.handleTokenAdd(textValue);
    }

    onBlur && onBlur();
  }

  onChange = (event, data) => {
    this.setState({ textValue: data.value.trim() });
  }

  onKeyDown = (event, data) => {
    const { textValue } = this.state;

    switch (keycode(event)) {
      case 'enter':
      case 'space':
        this.handleTokenAdd(textValue);
        break;
    }
  }
}
