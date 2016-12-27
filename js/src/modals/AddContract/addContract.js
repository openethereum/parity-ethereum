// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { newError } from '~/redux/actions';
import { Button, Modal, Form, Input, InputAddress, RadioButtons } from '~/ui';
import { AddIcon, CancelIcon, NextIcon, PrevIcon } from '~/ui/Icons';

import Store from './store';

const STEPS = [
  <FormattedMessage
    id='addContract.title.type'
    defaultMessage='choose a contract type'
    key='type' />,
  <FormattedMessage
    id='addContract.title.details'
    defaultMessage='enter contract details'
    key='details' />
];

@observer
export default class AddContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    contracts: PropTypes.object.isRequired,
    onClose: PropTypes.func
  };

  store = new Store(this.context.api);

  state = {
    abiParsed: null,
    step: 0
  };

  componentDidMount () {
    this.onChangeABIType(null, this.state.abiTypeIndex);
  }

  render () {
    const { step } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ step }
        steps={ STEPS }
        visible>
        { this.renderStep(step) }
      </Modal>
    );
  }

  renderStep (step) {
    switch (step) {
      case 0:
        return this.renderContractTypeSelector();

      default:
        return this.renderFields();
    }
  }

  renderContractTypeSelector () {
    const { abiTypeIndex } = this.state;

    return (
      <RadioButtons
        name='contractType'
        value={ abiTypeIndex }
        values={ this.getAbiTypes() }
        onChange={ this.onChangeABIType }
      />
    );
  }

  renderDialogActions () {
    const { addressError, nameError, step } = this.state;
    const hasError = !!(addressError || nameError);

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='addContract.button.cancel'
            defaultMessage='Cancel' />
        }
        onClick={ this.onClose } />
    );

    if (step === 0) {
      return [
        cancelBtn,
        <Button
          icon={ <NextIcon /> }
          key='next'
          label={
            <FormattedMessage
              id='addContract.button.next'
              defaultMessage='Next' />
          }
          onClick={ this.onNext } />
      ];
    }

    return [
      cancelBtn,
      <Button
        icon={ <PrevIcon /> }
        key='prev'
        label={
          <FormattedMessage
            id='addContract.button.prev'
            defaultMessage='Back' />
        }
        onClick={ this.onPrev } />,
      <Button
        icon={ <AddIcon /> }
        key='add'
        label={
          <FormattedMessage
            id='addContract.button.add'
            defaultMessage='Add Contract' />
        }
        disabled={ hasError }
        onClick={ this.onAdd } />
    ];
  }

  renderFields () {
    const { abi, abiError, address, addressError, description, name, nameError, abiType } = this.state;

    return (
      <Form>
        <InputAddress
          error={ addressError }
          hint={
            <FormattedMessage
              id='addContract.address.hint'
              defaultMessage='the network address for the contract' />
          }
          label={
            <FormattedMessage
              id='addContract.address.label'
              defaultMessage='network address' />
          }
          onChange={ this.onChangeAddress }
          onSubmit={ this.onEditAddress }
          value={ address } />
        <Input
          error={ nameError }
          hint={
            <FormattedMessage
              id='addContract.name.hint'
              defaultMessage='a descriptive name for the contract' />
          }
          label={
            <FormattedMessage
              id='addContract.name.label'
              defaultMessage='contract name' />
          }
          onSubmit={ this.onEditName }
          value={ name } />
        <Input
          hint={
            <FormattedMessage
              id='addContract.description.hint'
              defaultMessage='an expanded description for the entry' />
          }
          label={
            <FormattedMessage
              id='addContract.description.label'
              defaultMessage='(optional) contract description' />
          }
          onSubmit={ this.onEditDescription }
          value={ description } />
        <Input
          error={ abiError }
          hint={
            <FormattedMessage
              id='addContract.abi.hint'
              defaultMessage='the abi for the contract' />
          }
          label={
            <FormattedMessage
              id='addContract.abi.label'
              defaultMessage='contract abi' />
          }
          onSubmit={ this.onEditAbi }
          readOnly={ abiType.readOnly }
          value={ abi } />
      </Form>
    );
  }

  getAbiTypes () {
    return ABI_TYPES.map((type, index) => ({
      label: type.label,
      description: type.description,
      key: index,
      ...type
    }));
  }

  onNext = () => {
    this.setState({ step: this.state.step + 1 });
  }

  onPrev = () => {
    this.setState({ step: this.state.step - 1 });
  }

  onChangeABIType = (value, index) => {
    const abiType = value || ABI_TYPES[index];
    this.setState({ abiTypeIndex: index, abiType });
    this.onEditAbi(abiType.value);
  }

  onEditAbi = (abiIn) => {
    const { api } = this.context;
    const { abi, abiError, abiParsed } = validateAbi(abiIn, api);

    this.setState({ abi, abiError, abiParsed });
  }

  onChangeAddress = (event, value) => {
    this.onEditAddress(value);
  }

  onEditAddress = (_address) => {
    const { contracts } = this.props;
    let { address, addressError } = validateAddress(_address);

    if (!addressError) {
      const contract = contracts[address];

      if (contract) {
        addressError = ERRORS.duplicateAddress;
      }
    }

    this.setState({
      address,
      addressError
    });
  }

  onEditDescription = (description) => {
    this.setState({ description });
  }

  onEditName = (name) => {
    this.setState(validateName(name));
  }

  onAdd = () => {
    return this.store
      .addContract()
      .then(() => {
        this.onClose();
      })
      .catch((error) => {
        newError(error);
      });
  }

  onClose = () => {
    this.props.onClose();
  }
}
