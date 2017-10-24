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

import BigNumber from 'bignumber.js';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { Button, GasPriceEditor, IdentityIcon, Portal, Warning } from '~/ui';
import { CancelIcon } from '~/ui/Icons';
import { ERRORS, validateAbi, validateCode, validateName, validatePositiveNumber } from '~/util/validation';
import { deploy, deployEstimateGas, getSender, loadSender, setSender } from '~/util/tx';
import { setRequest } from '~/redux/providers/requestsActions';

import DetailsStep from './DetailsStep';
import ParametersStep from './ParametersStep';
import Extras from '../Transfer/Extras';

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
  },
  EXTRAS: {
    title: (
      <FormattedMessage
        id='deployContract.title.extras'
        defaultMessage='extra information'
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
    code: PropTypes.string,
    gasLimit: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired,
    onSetRequest: PropTypes.func.isRequired,
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
    amount: '0',
    amountValue: new BigNumber(0),
    amountError: '',
    code: '',
    codeError: ERRORS.invalidCode,
    description: '',
    descriptionError: null,
    extras: false,
    fromAddress: getSender() || Object.keys(this.props.accounts)[0],
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

    loadSender(this.context.api)
      .then((defaultAccount) => {
        if (defaultAccount !== this.state.fromAddress) {
          this.setState({ fromAddress: defaultAccount });
        }
      });
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

    const realStepKeys = Object.keys(STEPS)
        .filter((k) => {
          if (k === 'CONTRACT_PARAMETERS') {
            return inputs.length > 0;
          }

          if (k === 'EXTRAS') {
            return this.state.extras;
          }

          return true;
        });

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
    const { deployError, abiError, amountError, codeError, nameError, descriptionError, fromAddressError, fromAddress, step } = this.state;
    const isValid = !nameError && !fromAddressError && !descriptionError && !abiError && !codeError && !amountError;

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

    const closeBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='close'
        label={
          <FormattedMessage
            id='deployContract.button.close'
            defaultMessage='Close'
          />
        }
        onClick={ this.onClose }
      />
    );

    if (deployError) {
      return closeBtn;
    }

    const createButton = (
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
    );

    const nextButton = (
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
        onClick={ this.onNextStep }
      />
    );

    const hasParameters = this.state.inputs.length > 0;
    const showExtras = this.state.extras;

    switch (step) {
      case 'CONTRACT_DETAILS':
        return [
          cancelBtn,
          hasParameters || showExtras
            ? nextButton
            : createButton
        ];

      case 'CONTRACT_PARAMETERS':
        return [
          cancelBtn,
          showExtras
            ? nextButton
            : createButton
        ];

      case 'EXTRAS':
        return [
          cancelBtn,
          createButton
        ];
    }
  }

  renderStep () {
    const { accounts, readOnly } = this.props;
    const { step } = this.state;

    switch (step) {
      case 'CONTRACT_DETAILS':
        return (
          <DetailsStep
            { ...this.state }
            accounts={ accounts }
            onAmountChange={ this.onAmountChange }
            onExtrasChange={ this.onExtrasChange }
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

      case 'EXTRAS':
        return this.renderExtrasPage();
    }
  }

  renderExtrasPage () {
    if (!this.gasStore.histogram) {
      return null;
    }

    return (
      <Extras
        gasStore={ this.gasStore }
        hideData
        isEth
      />
    );
  }

  estimateGas = () => {
    const { api } = this.context;
    const { abiError, abiParsed, amountValue, amountError, code, codeError, fromAddress, fromAddressError, params } = this.state;

    if (abiError || codeError || fromAddressError || amountError) {
      return;
    }

    const options = {
      data: code,
      from: fromAddress,
      value: amountValue
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

  onNextStep = () => {
    switch (this.state.step) {
      case 'CONTRACT_DETAILS':
        return this.onParametersStep();

      case 'CONTRACT_PARAMETERS':
        return this.onExtrasStep();

      default:
        console.warn('wrong call of "onNextStep" from', this.state.step);
    }
  }

  onParametersStep = () => {
    const { inputs } = this.state;

    if (inputs.length) {
      return this.setState({ step: 'CONTRACT_PARAMETERS' });
    }

    return this.onExtrasStep();
  }

  onExtrasStep = () => {
    if (this.state.extras) {
      return this.setState({ step: 'EXTRAS' });
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

  onAmountChange = (amount) => {
    const { numberError } = validatePositiveNumber(amount);
    const nextAmountValue = numberError
      ? new BigNumber(0)
      : this.context.api.util.toWei(amount);

    this.gasStore.setEthValue(nextAmountValue);
    this.setState({ amount, amountValue: nextAmountValue, amountError: numberError }, this.estimateGas);
  }

  onExtrasChange = (extras) => {
    this.setState({ extras });
  }

  onDeployStart = () => {
    const { api } = this.context;
    const { source } = this.props;
    const { abiParsed, amountValue, code, description, name, params, fromAddress } = this.state;

    const metadata = {
      abi: abiParsed,
      contract: true,
      deleted: false,
      timestamp: Date.now(),
      name,
      description,
      source
    };

    const options = this.gasStore.overrideTransaction({
      data: code,
      from: fromAddress,
      value: amountValue
    });

    const contract = api.newContract(abiParsed);

    setSender(fromAddress);
    this.onClose();
    deploy(contract, options, params, true)
      .then((requestId) => {
        const requestMetadata = { ...metadata, deployment: true };

        this.props.onSetRequest(requestId, { metadata: requestMetadata }, false);
      });
  }

  onClose = () => {
    this.props.onClose();
  }
}

function mapStateToProps (state) {
  const { gasLimit } = state.nodeStatus;

  return {
    gasLimit
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onSetRequest: setRequest
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(DeployContract);
