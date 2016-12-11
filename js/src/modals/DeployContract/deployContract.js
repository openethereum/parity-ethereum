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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';
import { pick } from 'lodash';

import { BusyStep, CompletedStep, CopyToClipboard, Button, IdentityIcon, Modal, TxHash } from '~/ui';
import { ERRORS, validateAbi, validateCode, validateName } from '~/util/validation';

import DetailsStep from './DetailsStep';
import ParametersStep from './ParametersStep';
import ErrorStep from './ErrorStep';

import styles from './deployContract.css';

import { ERROR_CODES } from '~/api/transport/error';

const STEPS = {
  CONTRACT_DETAILS: { title: 'contract details' },
  CONTRACT_PARAMETERS: { title: 'contract parameters' },
  DEPLOYMENT: { title: 'deployment', waiting: true },
  COMPLETED: { title: 'completed' }
};

class DeployContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired,
    balances: PropTypes.object,
    abi: PropTypes.string,
    code: PropTypes.string,
    readOnly: PropTypes.bool,
    source: PropTypes.string
  };

  static defaultProps = {
    readOnly: false,
    source: ''
  };

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

    deployState: '',
    deployError: null,
    rejected: false,
    step: 'CONTRACT_DETAILS'
  }

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
    const { step, deployError, rejected, inputs } = this.state;

    const realStep = Object.keys(STEPS).findIndex((k) => k === step);
    const realSteps = deployError || rejected
      ? null
      : Object.keys(STEPS)
        .filter((k) => k !== 'CONTRACT_PARAMETERS' || inputs.length > 0)
        .map((k) => STEPS[k]);

    const title = realSteps
      ? null
      : (deployError ? 'deployment failed' : 'rejected');

    const waiting = realSteps
      ? realSteps.map((s, i) => s.waiting ? i : false).filter((v) => v !== false)
      : null;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ realStep }
        steps={ realSteps ? realSteps.map((s) => s.title) : null }
        title={ title }
        waiting={ waiting }
        visible
      >
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

    const closeBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Close'
        onClick={ this.onClose } />
    );

    const closeBtnOk = (
      <Button
        icon={ <ActionDoneAll /> }
        label='Close'
        onClick={ this.onClose } />
    );

    if (deployError) {
      return closeBtn;
    }

    switch (step) {
      case 'CONTRACT_DETAILS':
        return [
          cancelBtn,
          <Button
            disabled={ !isValid }
            icon={ <IdentityIcon button address={ fromAddress } /> }
            label='Next'
            onClick={ this.onParametersStep } />
        ];

      case 'CONTRACT_PARAMETERS':
        return [
          cancelBtn,
          <Button
            icon={ <IdentityIcon button address={ fromAddress } /> }
            label='Create'
            onClick={ this.onDeployStart } />
        ];

      case 'DEPLOYMENT':
        return [ closeBtn ];

      case 'COMPLETED':
        return [ closeBtnOk ];
    }
  }

  renderStep () {
    const { accounts, readOnly, balances } = this.props;
    const { address, deployError, step, deployState, txhash, rejected } = this.state;

    if (deployError) {
      return (
        <ErrorStep error={ deployError } />
      );
    }

    if (rejected) {
      return (
        <BusyStep
          title='The deployment has been rejected'
          state='You can safely close this window, the contract deployment will not occur.'
        />
      );
    }

    switch (step) {
      case 'CONTRACT_DETAILS':
        return (
          <DetailsStep
            { ...this.state }
            accounts={ accounts }
            balances={ balances }
            readOnly={ readOnly }
            onFromAddressChange={ this.onFromAddressChange }
            onDescriptionChange={ this.onDescriptionChange }
            onNameChange={ this.onNameChange }
            onAbiChange={ this.onAbiChange }
            onCodeChange={ this.onCodeChange }
            onParamsChange={ this.onParamsChange }
            onInputsChange={ this.onInputsChange }
          />
        );

      case 'CONTRACT_PARAMETERS':
        return (
          <ParametersStep
            { ...this.state }
            readOnly={ readOnly }
            accounts={ accounts }
            onParamsChange={ this.onParamsChange }
          />
        );

      case 'DEPLOYMENT':
        const body = txhash
          ? <TxHash hash={ txhash } />
          : null;
        return (
          <BusyStep
            title='The deployment is currently in progress'
            state={ deployState }>
            { body }
          </BusyStep>
        );

      case 'COMPLETED':
        return (
          <CompletedStep>
            <div>Your contract has been deployed at</div>
            <div>
              <CopyToClipboard data={ address } label='copy address to clipboard' />
              <IdentityIcon address={ address } inline center className={ styles.identityicon } />
              <div className={ styles.address }>{ address }</div>
            </div>
            <TxHash hash={ txhash } />
          </CompletedStep>
        );
    }
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
      : 'a valid account as the contract owner needs to be selected';

    this.setState({ fromAddress, fromAddressError });
  }

  onNameChange = (name) => {
    this.setState(validateName(name));
  }

  onParamsChange = (params) => {
    this.setState({ params });
  }

  onInputsChange = (inputs) => {
    this.setState({ inputs });
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
    const { api, store } = this.context;
    const { source } = this.props;
    const { abiParsed, code, description, name, params, fromAddress } = this.state;
    const options = {
      data: code,
      from: fromAddress
    };

    this.setState({ step: 'DEPLOYMENT' });

    api
      .newContract(abiParsed)
      .deploy(options, params, this.onDeploymentState)
      .then((address) => {
        return Promise.all([
          api.parity.setAccountName(address, name),
          api.parity.setAccountMeta(address, {
            abi: abiParsed,
            contract: true,
            timestamp: Date.now(),
            deleted: false,
            source,
            description
          })
        ])
        .then(() => {
          console.log(`contract deployed at ${address}`);
          this.setState({ step: 'DEPLOYMENT', address });
        });
      })
      .catch((error) => {
        if (error.code === ERROR_CODES.REQUEST_REJECTED) {
          this.setState({ rejected: true });
          return false;
        }

        console.error('error deploying contract', error);
        this.setState({ deployError: error });
        store.dispatch({ type: 'newError', error });
      });
  }

  onDeploymentState = (error, data) => {
    if (error) {
      console.error('onDeploymentState', error);
      return;
    }

    switch (data.state) {
      case 'estimateGas':
      case 'postTransaction':
        this.setState({ deployState: 'Preparing transaction for network transmission' });
        return;

      case 'checkRequest':
        this.setState({ deployState: 'Waiting for confirmation of the transaction in the Parity Secure Signer' });
        return;

      case 'getTransactionReceipt':
        this.setState({ deployState: 'Waiting for the contract deployment transaction receipt', txhash: data.txhash });
        return;

      case 'hasReceipt':
      case 'getCode':
        this.setState({ deployState: 'Validating the deployed contract code' });
        return;

      case 'completed':
        this.setState({ deployState: 'The contract deployment has been completed' });
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

function mapStateToProps (initState, initProps) {
  const fromAddresses = Object.keys(initProps.accounts);

  return (state) => {
    const balances = pick(state.balances.balances, fromAddresses);
    return { balances };
  };
}

export default connect(
  mapStateToProps
)(DeployContract);

