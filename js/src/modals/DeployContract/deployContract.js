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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { newError } from '../../redux/actions';
import { Button, IdentityIcon, Modal } from '../../ui';

import CodeStep from './CodeStep';
import DetailsStep from './DetailsStep';

const steps = ['contract details', 'interface & code', 'deployment', 'completed'];

class DeployContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    newError: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired
  }

  state = {
    abi: '',
    abiError: 'Invalid or empty ABI',
    code: '',
    codeError: 'Invalid or empty contract code',
    deployState: '',
    description: '',
    descriptionError: null,
    fromAddress: Object.keys(this.props.accounts)[0],
    fromAddressError: null,
    name: '',
    nameError: 'Contract name needs to be >2 charaters',
    step: 0
  }

  render () {
    const { step } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ step }
        steps={ steps }
        waiting={ [2] }
        visible>
        { this.renderStep() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { abiError, codeError, nameError, descriptionError, fromAddressError, fromAddress, step } = this.state;
    const isValidStep0 = !nameError && !fromAddressError && !descriptionError;
    const isValidStep1 = !abiError && !codeError;

    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose } />
    );

    switch (step) {
      case 0:
        return [
          cancelBtn,
          <Button
            disabled={ !isValidStep0 }
            icon={ <IdentityIcon button address={ fromAddress } /> }
            label='Next'
            onClick={ this.onNextStep } />
        ];

      case 1:
        return [
          cancelBtn,
          <Button
            disabled={ !isValidStep1 }
            icon={ <IdentityIcon button address={ fromAddress } /> }
            label='Create'
            onClick={ this.onDeployStart } />
        ];

      case 2:
        return [
          cancelBtn
        ];

      case 3:
        return [
          <Button
            icon={ <ActionDoneAll /> }
            label='Close'
            onClick={ this.onClose } />
        ];
    }
  }

  renderStep () {
    const { accounts } = this.props;
    const { abi, abiError, code, codeError, description, descriptionError, fromAddress, fromAddressError, name, nameError, step } = this.state;

    switch (step) {
      case 0:
        return (
          <DetailsStep
            accounts={ accounts }
            description={ description }
            descriptionError={ descriptionError }
            onDescriptionChange={ this.onDescriptionChange }
            fromAddress={ fromAddress }
            fromAddressError={ fromAddressError }
            onFromAddressChange={ this.onFromAddressChange }
            name={ name }
            nameError={ nameError }
            onNameChange={ this.onNameChange } />
        );

      case 1:
        return (
          <CodeStep
            abi={ abi }
            abiError={ abiError }
            onAbiChange={ this.onAbiChange }
            code={ code }
            codeError={ codeError }
            onCodeChange={ this.onCodeChange } />
        );
    }
  }

  onDescriptionChange = (description) => {
    this.setState({ description, descriptionError: null });
  }

  onFromAddressChange = (event, fromAddress) => {
    const { api } = this.context;
    const fromAddressError = api.util.isAddressValid(fromAddress)
      ? null
      : 'a valid account as the contract owner needs to be selected';

    this.setState({ fromAddress, fromAddressError });
  }

  onNameChange = (name) => {
    const nameError = name && name.length > 2
      ? null
      : 'specify a valid name, >2 characters';

    this.setState({ name, nameError });
  }

  onAbiChange = (abi) => {
    const { api } = this.context;

    try {
      const parsedAbi = JSON.parse(abi);

      if (!api.util.isArray(parsedAbi) && !parsedAbi.length) {
        throw new Error();
      }

      this.setState({ parsedAbi, abi, abiError: null });
    } catch (error) {
      console.error(error);
      this.setState({ abi, abiError: 'ABI needs to be a valid JSON array' });
    }
  }

  onCodeChange = (code) => {
    const { api } = this.context;
    const codeError = api.util.isHex(code)
      ? null
      : 'provide the valid compiled hex string of the contract code';

    this.setState({ code, codeError });
  }

  onDeployStart = () => {
    const { api } = this.context;
    const { newError } = this.props;
    const { parsedAbi, code, description, name, fromAddress } = this.state;
    const options = {
      data: code,
      from: fromAddress
    };

    this.setState({ step: 2 });

    api
      .newContract(parsedAbi)
      .deploy(options, null, this.onDeploymentState)
      .then((address) => {
        return Promise.all([
          api.personal.setAccountName(address, name),
          api.personal.setAccountMeta(address, {
            abi: parsedAbi,
            contract: true,
            deleted: false,
            description
          })
        ])
        .then(() => {
          console.log(`contract deployed at ${address}`);
        });
      })
      .catch((error) => {
        console.error('error deploying contract', error);
        newError(error);
      });
  }

  onDeploymentState = (error, data) => {
    if (error) {
      console.error('onDeploymentState', error);
      return;
    }

    console.log('onDeploymentState', data);

    switch (data.state) {
      case 'estimateGas':
      case 'postTransaction':
        this.setState({ deployState: 'Preparing transaction for network transmission' });
        return;

      case 'checkRequest':
        this.setState({ deployState: 'Waiting for confirmation of transaction in the Signer' });
        return;

      case 'getTransactionReceipt':
        this.setState({ deployState: 'Waiting for contract to be deployed/mined' });
        return;

      case 'hasReceipt':
      case 'getCode':
        this.setState({ deployState: 'Validating contract deployment' });
        return;

      case 'completed':
        this.setState({ deployState: 'Contract deployment completed' });
        return;

      default:
        console.error('Unknow contract deployment state', data);
        return;
    }
  }

  onNextStep = () => {
    this.setState({
      step: this.state.step + 1
    });
  }

  onClose = () => {
    this.props.onClose();
  }
}

function mapStateToProps (state) {
  return {};
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ newError }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(DeployContract);
