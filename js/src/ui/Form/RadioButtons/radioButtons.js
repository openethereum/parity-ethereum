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

import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';
import React, { Component, PropTypes } from 'react';

import { arrayOrObjectProptype } from '~/util/proptypes';

import Label from '../Label';
import styles from './radioButtons.css';

export default class RadioButtons extends Component {
  static propTypes = {
    className: PropTypes.string,
    label: PropTypes.node,
    name: PropTypes.string,
    onChange: PropTypes.func.isRequired,
    value: PropTypes.any,
    values: arrayOrObjectProptype().isRequired
  };

  static defaultProps = {
    value: 0,
    name: ''
  };

  render () {
    const { className, label, value, values } = this.props;

    const index = Number.isNaN(parseInt(value))
      ? values.findIndex((_value) => _value.key === value)
      : parseInt(value);
    const selectedValue = typeof value !== 'object'
      ? values[index]
      : value;
    const key = this.getKey(selectedValue, index);

    return (
      <div className={ [styles.container, className].join(' ') }>
        <Label
          className={ styles.label }
          label={ label }
        />
        <RadioButtonGroup
          name={ name }
          onChange={ this.onChange }
          valueSelected={ key }
        >
          { this.renderContent() }
        </RadioButtonGroup>
      </div>
    );
  }

  renderContent () {
    const { values } = this.props;

    return values.map((value, index) => {
      const label = typeof value === 'string'
        ? value
        : value.label || '';
      const description = (typeof value !== 'string' && value.description) || null;
      const key = this.getKey(value, index);

      return (
        <RadioButton
          className={ styles.radioButton }
          key={ index }
          label={
            <div className={ styles.radioLabel }>
              <span>{ label }</span>
              {
                description
                ? <span className={ styles.description }>{ description }</span>
                : null
              }
            </div>
          }
          value={ key }
        />
      );
    });
  }

  getKey (value, index) {
    if (typeof value !== 'string') {
      return typeof value.key === 'undefined'
        ? index
        : value.key;
    }

    return index;
  }

  onChange = (event, index) => {
    const { onChange, values } = this.props;
    const value = values[index] || values.find((value) => value.key === index);

    onChange(value, index);
  }
}
