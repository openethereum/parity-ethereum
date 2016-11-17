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
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';
import NavigationArrowBack from 'material-ui/svg-icons/navigation/arrow-back';

import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';

import { Button, Modal, Form, Input, InputAddress } from '../../ui';
import { ERRORS, validateAbi, validateAddress, validateName } from '../../util/validation';

import { eip20, wallet } from '../../contracts/abi';
import styles from './addContract.css';

const ABI_TYPES = [
  {
    label: 'Token', readOnly: true, value: JSON.stringify(eip20),
    type: 'token',
    description: (<span>A standard <a href='https://github.com/ethereum/EIPs/issues/20' target='_blank'>ERC 20</a> token</span>)
  },
  {
    label: 'Multisig Wallet', readOnly: true,
    type: 'multisig',
    value: JSON.stringify(wallet),
    description: (<span>Official Multisig contract: <a href='https://github.com/ethereum/dapp-bin/blob/master/wallet/wallet.sol' target='_blank'>see contract code</a></span>)
  },
  {
    label: 'Custom Contract', value: '',
    type: 'custom',
    description: 'Contract created from custom ABI'
  }
];

const STEPS = [ 'choose a contract type', 'enter contract details' ];

export default class AddContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    contracts: PropTypes.object.isRequired,
    onClose: PropTypes.func
  };

  state = {
    abi: '',
    abiError: ERRORS.invalidAbi,
    abiType: ABI_TYPES[2],
    abiTypeIndex: 2,
    abiParsed: null,
    address: '',
    addressError: ERRORS.invalidAddress,
    name: '',
    nameError: ERRORS.invalidName,
    description: '',
    step: 0
  };

  componentDidMount () {
    this.onChangeABIType(null, this.state.abiTypeIndex);
  }

  render () {
    const { step } = this.state;

    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        steps={ STEPS }
        current={ step }
      >
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
      <RadioButtonGroup
        valueSelected={ abiTypeIndex }
        name='contractType'
        onChange={ this.onChangeABIType }
      >
        { this.renderAbiTypes() }
      </RadioButtonGroup>
    );
  }

  renderDialogActions () {
    const { addressError, nameError, step } = this.state;
    const hasError = !!(addressError || nameError);

    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose } />
    );

    if (step === 0) {
      const nextBtn = (
        <Button
          icon={ <NavigationArrowForward /> }
          label='Next'
          onClick={ this.onNext } />
      );

      return [ cancelBtn, nextBtn ];
    }

    const prevBtn = (
      <Button
        icon={ <NavigationArrowBack /> }
        label='Back'
        onClick={ this.onPrev } />
    );

    const addBtn = (
      <Button
        icon={ <ContentAdd /> }
        label='Add Contract'
        disabled={ hasError }
        onClick={ this.onAdd } />
    );

    return [ cancelBtn, prevBtn, addBtn ];
  }

  renderFields () {
    const { abi, abiError, address, addressError, description, name, nameError, abiType } = this.state;

    return (
      <Form>
        <InputAddress
          label='network address'
          hint='the network address for the contract'
          error={ addressError }
          value={ address }
          onSubmit={ this.onEditAddress }
          onChange={ this.onChangeAddress }
        />
        <Input
          label='contract name'
          hint='a descriptive name for the contract'
          error={ nameError }
          value={ name }
          onSubmit={ this.onEditName } />
        <Input
          multiLine
          rows={ 1 }
          label='(optional) contract description'
          hint='an expanded description for the entry'
          value={ description }
          onSubmit={ this.onEditDescription } />

        <Input
          label='contract abi'
          hint='the abi for the contract'
          error={ abiError }
          value={ abi }
          readOnly={ abiType.readOnly }
          onSubmit={ this.onEditAbi }
        />
      </Form>
    );
  }

  renderAbiTypes () {
    return ABI_TYPES.map((type, index) => (
      <RadioButton
        className={ styles.spaced }
        value={ index }
        label={ (
          <div className={ styles.typeContainer }>
            <span>{ type.label }</span>
            <span className={ styles.desc }>{ type.description }</span>
          </div>
        ) }
        key={ index }
      />
    ));
  }

  onNext = () => {
    this.setState({ step: this.state.step + 1 });
  }

  onPrev = () => {
    this.setState({ step: this.state.step - 1 });
  }

  onChangeABIType = (event, index) => {
    const abiType = ABI_TYPES[index];
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

      if (contract && !contract.meta.deleted) {
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
    const { api } = this.context;
    const { abiParsed, address, name, description, abiType } = this.state;

    Promise.all([
      api.parity.setAccountName(address, name),
      api.parity.setAccountMeta(address, {
        contract: true,
        deleted: false,
        timestamp: Date.now(),
        abi: abiParsed,
        type: abiType.type,
        description
      })
    ]).catch((error) => {
      console.error('onAdd', error);
    });

    this.props.onClose();
  }

  onClose = () => {
    this.props.onClose();
  }
}
