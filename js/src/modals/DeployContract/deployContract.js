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

import { pick } from 'lodash';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { Button, GasPriceEditor, IdentityIcon, Portal, Warning } from '~/ui';
import { CancelIcon } from '~/ui/Icons';
import { ERRORS, validateAbi, validateCode, validateName } from '~/util/validation';
import { deploy, deployEstimateGas } from '~/util/tx';

import DetailsStep from './DetailsStep';
import ParametersStep from './ParametersStep';

const STEPS = {
  CONTRACT_DETAILS: {
    title: (
      <FormattedMessage
        id='deployContract.title.details'
        defaultMessage='contract details'
      />
    )
  },
  CONTRACT_PARAMETERS: {
    title: (
      <FormattedMessage
        id='deployContract.title.parameters'
        defaultMessage='contract parameters'
      />
    )
  }
};

@observer
class DeployContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    abi: PropTypes.string,
    balances: PropTypes.object,
    code: PropTypes.string,
    gasLimit: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired,
    readOnly: PropTypes.bool,
    source: PropTypes.string
  };

  static defaultProps = {
    readOnly: false,
    source: ''
  };

  gasStore = new GasPriceEditor.Store(this.context.api, { gasLimit: this.props.gasLimit });

  state = {
    abi: '',
    abiError: ERRORS.invalidAbi,
    code: '',
    codeError: ERRORS.invalidCode,
    description: '',
    descriptionError: null,
    fromAddress: Object.keys(this.props.accounts)[0],
    fromAddressError: null,
    name: '',
    nameError: ERRORS.invalidName,
    params: [],
    paramsError: [],
    inputs: [],
    step: 'CONTRACT_DETAILS'
  };

  componentWillMount () {
    const { abi, code } = this.props;

    if (abi && code) {
      this.setState({ abi, code });
    }
  }

  componentWillReceiveProps (nextProps) {
    const { abi, code } = nextProps;
    const newState = {};

    if (abi !== this.props.abi) {
      newState.abi = abi;
    }

    if (code !== this.props.code) {
      newState.code = code;
    }

    if (Object.keys(newState).length) {
      this.setState(newState);
    }
  }

  render () {
    const { step, inputs } = this.state;

    const realStepKeys = Object.keys(STEPS).filter((k) => k !== 'CONTRACT_PARAMETERS' || inputs.length > 0);
    const realStep = realStepKeys.findIndex((k) => k === step);
    const realSteps = realStepKeys.map((k) => STEPS[k]);

    return (
      <Portal
        buttons={ this.renderDialogActions() }
        activeStep={ realStep }
        onClose={ this.onClose }
        open
        steps={ realSteps.map((s) => s.title) }
      >
        { this.renderExceptionWarning() }
        { this.renderStep() }
      </Portal>
    );
  }

  renderExceptionWarning () {
    const { step } = this.state;
    const { errorEstimated } = this.gasStore;
    const realStep = Object.keys(STEPS).findIndex((k) => k === step);

    if (!errorEstimated || realStep >= 2) {
      return null;
    }

    return (
      <Warning warning={ errorEstimated } />
    );
  }

  renderDialogActions () {
    const { abiError, codeError, nameError, descriptionError, fromAddressError, fromAddress, step } = this.state;
    const isValid = !nameError && !fromAddressError && !descriptionError && !abiError && !codeError;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='deployContract.button.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.onClose }
      />
    );

    switch (step) {
      case 'CONTRACT_DETAILS':
        return [
          cancelBtn,
          <Button
            disabled={ !isValid }
            key='next'
            icon={
              <IdentityIcon
                address={ fromAddress }
                button
              />
            }
            label={
              <FormattedMessage
                id='deployContract.button.next'
                defaultMessage='Next'
              />
            }
            onClick={ this.onParametersStep }
          />
        ];

      case 'CONTRACT_PARAMETERS':
        return [
          cancelBtn,
          <Button
            icon={
              <IdentityIcon
                address={ fromAddress }
                button
              />
            }
            key='create'
            label={
              <FormattedMessage
                id='deployContract.button.create'
                defaultMessage='Create'
              />
            }
            onClick={ this.onDeployStart }
          />
        ];
    }
  }

  renderStep () {
    const { accounts, readOnly, balances } = this.props;
    const { step } = this.state;

    switch (step) {
      case 'CONTRACT_DETAILS':
        return (
          <DetailsStep
            { ...this.state }
            accounts={ accounts }
            balances={ balances }
            onFromAddressChange={ this.onFromAddressChange }
            onDescriptionChange={ this.onDescriptionChange }
            onNameChange={ this.onNameChange }
            onAbiChange={ this.onAbiChange }
            onCodeChange={ this.onCodeChange }
            onParamsChange={ this.onParamsChange }
            onInputsChange={ this.onInputsChange }
            readOnly={ readOnly }
          />
        );

      case 'CONTRACT_PARAMETERS':
        return (
          <ParametersStep
            { ...this.state }
            accounts={ accounts }
            onParamsChange={ this.onParamsChange }
            readOnly={ readOnly }
          />
        );
    }
  }

  estimateGas = () => {
    const { api } = this.context;
    const { abiError, abiParsed, code, codeError, fromAddress, fromAddressError, params } = this.state;

    if (abiError || codeError || fromAddressError) {
      return;
    }

    const options = {
      data: code,
      from: fromAddress
    };

    const contract = api.newContract(abiParsed);

    deployEstimateGas(contract, options, params)
      .then(([gasEst, gas]) => {
        this.gasStore.setEstimated(gasEst.toFixed(0));
        this.gasStore.setGas(gas.toFixed(0));
      })
      .catch((error) => {
        this.gasStore.setEstimatedError();
        console.warn('estimateGas', error);
      });
  }

  onParametersStep = () => {
    const { inputs } = this.state;

    if (inputs.length) {
      return this.setState({ step: 'CONTRACT_PARAMETERS' });
    }

    return this.onDeployStart();
  }

  onDescriptionChange = (description) => {
    this.setState({ description, descriptionError: null });
  }

  onFromAddressChange = (fromAddress) => {
    const { api } = this.context;

    const fromAddressError = api.util.isAddressValid(fromAddress)
      ? null
      : (
        <FormattedMessage
          id='deployContract.owner.noneSelected'
          defaultMessage='a valid account as the contract owner needs to be selected'
        />
      );

    this.setState({ fromAddress, fromAddressError }, this.estimateGas);
  }

  onNameChange = (name) => {
    this.setState(validateName(name));
  }

  onParamsChange = (params) => {
    this.setState({ params }, this.estimateGas);
  }

  onInputsChange = (inputs) => {
    this.setState({ inputs }, this.estimateGas);
  }

  onAbiChange = (abi) => {
    const { api } = this.context;

    this.setState(validateAbi(abi, api), this.estimateGas);
  }

  onCodeChange = (code) => {
    this.setState(validateCode(code), this.estimateGas);
  }

  onDeployStart = () => {
    const { api } = this.context;
    const { source } = this.props;
    const { abiParsed, code, description, name, params, fromAddress } = this.state;

    const metadata = {
      abi: abiParsed,
      contract: true,
      deleted: false,
      timestamp: Date.now(),
      name,
      description,
      source
    };

    const options = {
      data: code,
      from: fromAddress
    };

    const contract = api.newContract(abiParsed);

    deploy(contract, options, params, metadata)
      .then((address) => {
        // No contract address given, might need some confirmations
        // from the wallet owners...
        if (!address || /^(0x)?0*$/.test(address)) {
          return false;
        }

        metadata.blockNumber = contract._receipt
          ? contract.receipt.blockNumber.toNumber()
          : null;

        return Promise.all([
          api.parity.setAccountName(address, name),
          api.parity.setAccountMeta(address, metadata)
        ]);
      });

    this.onClose();
  }

  onClose = () => {
    this.props.onClose();
  }
}

function mapStateToProps (initState, initProps) {
  const { accounts } = initProps;

  const fromAddresses = Object.keys(accounts);

  return (state) => {
    const balances = pick(state.balances.balances, fromAddresses);
    const { gasLimit } = state.nodeStatus;

    return {
      accounts,
      balances,
      gasLimit
    };
  };
}

export default connect(
  mapStateToProps
)(DeployContract);
