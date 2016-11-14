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

import { AddressSelect, Form, Input, InputAddressSelect, Select } from '../../../ui';
import { validateAbi } from '../../../util/validation';

import styles from '../deployContract.css';

class TypedInput extends Component {

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

    if (type === ARRAY_TYPE) {
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
    const style = {
      width: 16,
      height: 16
    };

    return (
      <div>
        <IconButton
          iconStyle={ style }
          style={ style }
          onClick={ this.onAddField }
        >
          <AddIcon />
        </IconButton>

        <IconButton
          iconStyle={ style }
          style={ style }
          onClick={ this.onRemoveField }
        >
          <RemoveIcon />
        </IconButton>
      </div>
    );
  }

  renderType (type) {
    if (type === ADDRESS_TYPE) {
      return this.renderAddress();
    }

    if (type === BOOL_TYPE) {
      return this.renderBoolean();
    }

    if (type === STRING_TYPE) {
      return this.renderDefault();
    }

    if (type === BYTES_TYPE) {
      return this.renderDefault();
    }

    if (type === INT_TYPE) {
      return this.renderNumber();
    }

    if (type === FIXED_TYPE) {
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

export default class DetailsStep extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    abi: PropTypes.string,
    abiError: PropTypes.string,
    code: PropTypes.string,
    codeError: PropTypes.string,
    description: PropTypes.string,
    descriptionError: PropTypes.string,
    fromAddress: PropTypes.string,
    fromAddressError: PropTypes.string,
    name: PropTypes.string,
    nameError: PropTypes.string,
    params: PropTypes.array,
    paramsError: PropTypes.array,
    onAbiChange: PropTypes.func.isRequired,
    onCodeChange: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired,
    onDescriptionChange: PropTypes.func.isRequired,
    onNameChange: PropTypes.func.isRequired,
    onParamsChange: PropTypes.func.isRequired,
    readOnly: PropTypes.bool
  };

  static defaultProps = {
    readOnly: false
  };

  state = {
    inputs: []
  }

  componentDidMount () {
    const { abi, code } = this.props;

    if (abi) {
      this.onAbiChange(abi);
    }

    if (code) {
      this.onCodeChange(code);
    }
  }

  render () {
    const { accounts } = this.props;
    const { abi, abiError, code, codeError, fromAddress, fromAddressError, name, nameError, readOnly } = this.props;

    return (
      <Form>
        <AddressSelect
          label='from account (contract owner)'
          hint='the owner account for this contract'
          value={ fromAddress }
          error={ fromAddressError }
          accounts={ accounts }
          onChange={ this.onFromAddressChange } />
        <Input
          label='contract name'
          hint='a name for the deployed contract'
          error={ nameError }
          value={ name }
          onSubmit={ this.onNameChange } />
        <Input
          label='abi'
          hint='the abi of the contract to deploy'
          error={ abiError }
          value={ abi }
          onSubmit={ this.onAbiChange }
          readOnly={ readOnly } />
        <Input
          label='code'
          hint='the compiled code of the contract to deploy'
          error={ codeError }
          value={ code }
          onSubmit={ this.onCodeChange }
          readOnly={ readOnly } />
        { this.renderConstructorInputs() }
      </Form>
    );
  }

  renderConstructorInputs () {
    const { accounts, params, paramsError } = this.props;
    const { inputs } = this.state;

    if (!inputs || !inputs.length) {
      return null;
    }

    return inputs.map((input, index) => {
      const onChange = (value) => this.onParamChange(index, value);

      const label = `${input.name ? `${input.name}: ` : ''}${input.type}`;
      const value = params[index];
      const error = paramsError[index];
      const param = parseAbiType(input.type);

      return (
        <div key={ index } className={ styles.funcparams }>
          <TypedInput
            label={ label }
            value={ value }
            error={ error }
            accounts={ accounts }
            onChange={ onChange }
            param={ param }
          />
        </div>
      );
    });
  }

  onFromAddressChange = (event, fromAddress) => {
    const { onFromAddressChange } = this.props;

    onFromAddressChange(fromAddress);
  }

  onNameChange = (name) => {
    const { onNameChange } = this.props;

    onNameChange(name);
  }

  onParamChange = (index, value) => {
    const { params, onParamsChange } = this.props;

    params[index] = value;
    onParamsChange(params);
  }

  onAbiChange = (abi) => {
    const { api } = this.context;
    const { onAbiChange, onParamsChange } = this.props;
    const { abiError, abiParsed } = validateAbi(abi, api);

    if (!abiError) {
      const { inputs } = abiParsed
        .find((method) => method.type === 'constructor') || { inputs: [] };

      const params = [];

      inputs.forEach((input) => {
        const param = parseAbiType(input.type);
        params.push(param.default);
      });

      onParamsChange(params);
      this.setState({ inputs });
    } else {
      onParamsChange([]);
      this.setState({ inputs: [] });
    }

    onAbiChange(abi);
  }

  onCodeChange = (code) => {
    const { onCodeChange } = this.props;

    onCodeChange(code);
  }
}

const ARRAY_TYPE = 'ARRAY_TYPE';
const ADDRESS_TYPE = 'ADDRESS_TYPE';
const STRING_TYPE = 'STRING_TYPE';
const BOOL_TYPE = 'BOOL_TYPE';
const BYTES_TYPE = 'BYTES_TYPE';
const INT_TYPE = 'INT_TYPE';
const FIXED_TYPE = 'FIXED_TYPE';

function parseAbiType (type) {
  const arrayRegex = /^(.+)\[(\d*)\]$/;

  if (arrayRegex.test(type)) {
    const matches = arrayRegex.exec(type);

    const subtype = parseAbiType(matches[1]);
    const M = parseInt(matches[2]) || null;
    const defaultValue = !M
      ? []
      : range(M).map(() => subtype.default);

    return {
      type: ARRAY_TYPE,
      subtype: subtype,
      length: M,
      default: defaultValue
    };
  }

  const lengthRegex = /^(u?int|bytes)(\d{1,3})$/;

  if (lengthRegex.test(type)) {
    const matches = lengthRegex.exec(type);

    const subtype = parseAbiType(matches[1]);
    const length = parseInt(matches[2]);

    return {
      ...subtype,
      length
    };
  }

  const fixedLengthRegex = /^(u?fixed)(\d{1,3})x(\d{1,3})$/;

  if (fixedLengthRegex.test(type)) {
    const matches = fixedLengthRegex.exec(type);

    const subtype = parseAbiType(matches[1]);
    const M = parseInt(matches[2]);
    const N = parseInt(matches[3]);

    return {
      ...subtype,
      M, N
    };
  }

  if (type === 'string') {
    return {
      type: STRING_TYPE,
      default: ''
    };
  }

  if (type === 'bool') {
    return {
      type: BOOL_TYPE,
      default: false
    };
  }

  if (type === 'address') {
    return {
      type: ADDRESS_TYPE,
      default: '0x'
    };
  }

  if (type === 'bytes') {
    return {
      type: BYTES_TYPE,
      default: '0x'
    };
  }

  if (type === 'uint') {
    return {
      type: INT_TYPE,
      default: 0,
      length: 256,
      signed: false
    };
  }

  if (type === 'int') {
    return {
      type: INT_TYPE,
      default: 0,
      length: 256,
      signed: true
    };
  }

  if (type === 'ufixed') {
    return {
      type: FIXED_TYPE,
      default: 0,
      length: 256,
      signed: false
    };
  }

  if (type === 'fixed') {
    return {
      type: FIXED_TYPE,
      default: 0,
      length: 256,
      signed: true
    };
  }
}
