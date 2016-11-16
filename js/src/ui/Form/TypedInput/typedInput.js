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
import { MenuItem } from 'material-ui';
import { range } from 'lodash';

import IconButton from 'material-ui/IconButton';
import AddIcon from 'material-ui/svg-icons/content/add';
import RemoveIcon from 'material-ui/svg-icons/content/remove';

import { Input, InputAddressSelect, Select } from '../../../ui';
import { ABI_TYPES } from '../../../util/abi';

import styles from './typedInput.css';

export default class TypedInput extends Component {

  static propTypes = {
    onChange: PropTypes.func.isRequired,
    accounts: PropTypes.object.isRequired,
    param: PropTypes.object.isRequired,

    error: PropTypes.any,
    value: PropTypes.any,
    label: PropTypes.string
  };

  render () {
    const { param } = this.props;
    const { type } = param;

    if (type === ABI_TYPES.ARRAY) {
      const { accounts, label, value = param.default } = this.props;
      const { subtype, length } = param;

      const fixedLength = !!length;

      const inputs = range(length || value.length).map((_, index) => {
        const onChange = (inputValue) => {
          const newValues = [].concat(this.props.value);
          newValues[index] = inputValue;
          this.props.onChange(newValues);
        };

        return (
          <TypedInput
            key={ `${subtype.type}_${index}` }
            onChange={ onChange }
            accounts={ accounts }
            param={ subtype }
            value={ value[index] }
          />
        );
      });

      return (
        <div className={ styles.inputs }>
          <label>{ label }</label>
          { fixedLength ? null : this.renderLength() }
          { inputs }
        </div>
      );
    }

    return this.renderType(type);
  }

  renderLength () {
    const iconStyle = {
      width: 16,
      height: 16
    };

    const style = {
      width: 32,
      height: 32,
      padding: 0
    };

    return (
      <div>
        <IconButton
          iconStyle={ iconStyle }
          style={ style }
          onClick={ this.onAddField }
        >
          <AddIcon />
        </IconButton>

        <IconButton
          iconStyle={ iconStyle }
          style={ style }
          onClick={ this.onRemoveField }
        >
          <RemoveIcon />
        </IconButton>
      </div>
    );
  }

  renderType (type) {
    if (type === ABI_TYPES.ADDRESS) {
      return this.renderAddress();
    }

    if (type === ABI_TYPES.BOOL) {
      return this.renderBoolean();
    }

    if (type === ABI_TYPES.STRING) {
      return this.renderDefault();
    }

    if (type === ABI_TYPES.BYTES) {
      return this.renderDefault();
    }

    if (type === ABI_TYPES.INT) {
      return this.renderNumber();
    }

    if (type === ABI_TYPES.FIXED) {
      return this.renderNumber();
    }

    return this.renderDefault();
  }

  renderNumber () {
    const { label, value, error, param } = this.props;

    return (
      <Input
        label={ label }
        value={ value }
        error={ error }
        onSubmit={ this.onSubmit }
        type='number'
        min={ param.signed ? null : 0 }
      />
    );
  }

  renderDefault () {
    const { label, value, error } = this.props;

    return (
      <Input
        label={ label }
        value={ value }
        error={ error }
        onSubmit={ this.onSubmit }
      />
    );
  }

  renderAddress () {
    const { accounts, label, value, error } = this.props;

    return (
      <InputAddressSelect
        accounts={ accounts }
        label={ label }
        value={ value }
        error={ error }
        onChange={ this.onChange }
        editing
      />
    );
  }

  renderBoolean () {
    const { label, value, error } = this.props;

    const boolitems = ['false', 'true'].map((bool) => {
      return (
        <MenuItem
          key={ bool }
          value={ bool }
          label={ bool }
        >
          { bool }
        </MenuItem>
      );
    });

    return (
      <Select
        label={ label }
        value={ value ? 'true' : 'false' }
        error={ error }
        onChange={ this.onChangeBool }
      >
        { boolitems }
      </Select>
    );
  }

  onChangeBool = (event, _index, value) => {
    this.props.onChange(value === 'true');
  }

  onChange = (event, value) => {
    this.props.onChange(value);
  }

  onSubmit = (value) => {
    this.props.onChange(value);
  }

  onAddField = () => {
    const { value, onChange, param } = this.props;
    const newValues = [].concat(value, param.subtype.default);

    onChange(newValues);
  }

  onRemoveField = () => {
    const { value, onChange } = this.props;
    const newValues = value.slice(0, -1);

    onChange(newValues);
  }

}
