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
import { ERRORS, validateAbi, validateCode, validateName } from '../../util/validation';

import BusyStep from './BusyStep';
import CompletedStep from './CompletedStep';
import DetailsStep from './DetailsStep';
import ErrorStep from './ErrorStep';

const steps = ['contract details', 'deployment', 'completed'];

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
    abiError: ERRORS.invalidAbi,
    code: '',
    codeError: ERRORS.invalidCode,
    deployState: '',
    description: '',
    descriptionError: null,
    fromAddress: Object.keys(this.props.accounts)[0],
    fromAddressError: null,
    name: '',
    nameError: ERRORS.invalidName,
    step: 0,
    deployError: null
  }

  render () {
    const { step, deployError } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ step }
        steps={ deployError ? null : steps }
        title={ deployError ? 'deployment failed' : null }
        waiting={ [1] }
        visible>
        { this.renderStep() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { deployError, abiError, codeError, nameError, descriptionError, fromAddressError, fromAddress, step } = this.state;
    const isValid = !nameError && !fromAddressError && !descriptionError && !abiError && !codeError;

    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose } />
    );

    if (deployError) {
      return cancelBtn;
    }

    switch (step) {
      case 0:
        return [
          cancelBtn,
          <Button
            disabled={ !isValid }
            icon={ <IdentityIcon button address={ fromAddress } /> }
            label='Create'
            onClick={ this.onDeployStart } />
        ];

      case 1:
        return [
          cancelBtn
        ];

      case 2:
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
    const { deployError, step } = this.state;

    if (deployError) {
      return (
        <ErrorStep error={ deployError } />
      );
    }

    switch (step) {
      case 0:
        return (
          <DetailsStep
            { ...this.state }
            accounts={ accounts }
            onAbiChange={ this.onAbiChange }
            onCodeChange={ this.onCodeChange }
            onFromAddressChange={ this.onFromAddressChange }
            onDescriptionChange={ this.onDescriptionChange }
            onNameChange={ this.onNameChange } />
        );

      case 1:
        return (
          <BusyStep { ...this.state } />
        );

      case 2:
        return (
          <CompletedStep { ...this.state } />
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
    this.setState(validateName(name));
  }

  onAbiChange = (abi) => {
    const { api } = this.context;

    this.setState(validateAbi(abi, api));
  }

  onCodeChange = (code) => {
    const { api } = this.context;

    this.setState(validateCode(code, api));
  }

  onDeployStart = () => {
    const { api } = this.context;
    const { newError } = this.props;
    const { parsedAbi, code, description, name, fromAddress } = this.state;
    const options = {
      data: code,
      from: fromAddress
    };

    this.setState({ step: 1 });

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
          this.setState({ step: 2, address });
        });
      })
      .catch((deployError) => {
        console.error('error deploying contract', deployError);
        this.setState({ deployError });
        newError(deployError);
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
        this.setState({ deployState: 'Preparing transaction for network transmission', showSigner: false });
        return;

      case 'checkRequest':
        this.setState({ deployState: 'Waiting for confirmation of the transaction in the Signer', showSigner: true });
        return;

      case 'getTransactionReceipt':
        this.setState({ deployState: 'Waiting for contract to be deployed/mined', showSigner: false });
        return;

      case 'hasReceipt':
      case 'getCode':
        this.setState({ deployState: 'Validating contract deployment', showSigner: false });
        return;

      case 'completed':
        this.setState({ deployState: 'Contract deployment completed', showSigner: false });
        return;

      default:
        console.error('Unknow contract deployment state', data);
        return;
    }
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
