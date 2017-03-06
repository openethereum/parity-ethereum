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

import { BusyStep, Button, CompletedStep, CopyToClipboard, GasPriceEditor, IdentityIcon, Portal, TxHash, Warning } from '~/ui';
import { CancelIcon, DoneIcon } from '~/ui/Icons';
import { ERRORS, validateAbi, validateCode, validateName } from '~/util/validation';
import { deploy, deployEstimateGas } from '~/util/tx';

import DetailsStep from './DetailsStep';
import ParametersStep from './ParametersStep';
import ErrorStep from './ErrorStep';

import styles from './deployContract.css';

import { ERROR_CODES } from '~/api/transport/error';

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
  DEPLOYMENT: {
    waiting: true,
    title: (
      <FormattedMessage
        id='deployContract.title.deployment'
        defaultMessage='deployment'
      />
    )
  },
  COMPLETED: {
    title: (
      <FormattedMessage
        id='deployContract.title.completed'
        defaultMessage='completed'
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
    deployState: '',
    deployError: null,
    description: '',
    descriptionError: null,
    fromAddress: Object.keys(this.props.accounts)[0],
    fromAddressError: null,
    name: '',
    nameError: ERRORS.invalidName,
    params: [],
    paramsError: [],
    inputs: [],
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
      : (
        deployError
          ? (
            <FormattedMessage
              id='deployContract.title.failed'
              defaultMessage='deployment failed'
            />
          )
          : (
            <FormattedMessage
              id='deployContract.title.rejected'
              defaultMessage='rejected'
            />
          )
      );

    const waiting = realSteps
      ? realSteps.map((s, i) => s.waiting ? i : false).filter((v) => v !== false)
      : null;

    return (
      <Portal
        buttons={ this.renderDialogActions() }
        activeStep={ realStep }
        busySteps={ waiting }
        onClose={ this.onClose }
        open
        steps={
          realSteps
            ? realSteps.map((s) => s.title)
            : null
        }
        title={ title }
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
    const { deployError, abiError, codeError, nameError, descriptionError, fromAddressError, fromAddress, step } = this.state;
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

    const closeBtnOk = (
      <Button
        icon={ <DoneIcon /> }
        key='done'
        label={
          <FormattedMessage
            id='deployContract.button.done'
            defaultMessage='Done'
          />
        }
        onClick={ this.onClose }
      />
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
          title={
            <FormattedMessage
              id='deployContract.rejected.title'
              defaultMessage='The deployment has been rejected'
            />
          }
          state={
            <FormattedMessage
              id='deployContract.rejected.description'
              defaultMessage='You can safely close this window, the contract deployment will not occur.'
            />
          }
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

      case 'DEPLOYMENT':
        const body = txhash
          ? <TxHash hash={ txhash } />
          : null;

        return (
          <BusyStep
            title={
              <FormattedMessage
                id='deployContract.busy.title'
                defaultMessage='The deployment is currently in progress'
              />
            }
            state={ deployState }
          >
            { body }
          </BusyStep>
        );

      case 'COMPLETED':
        return (
          <CompletedStep>
            <div>
              <FormattedMessage
                id='deployContract.completed.description'
                defaultMessage='Your contract has been deployed at'
              />
            </div>
            <div>
              <CopyToClipboard data={ address } />
              <IdentityIcon
                address={ address }
                center
                className={ styles.identityicon }
                inline
              />
              <div className={ styles.address }>
                { address }
              </div>
            </div>
            <TxHash hash={ txhash } />
          </CompletedStep>
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
    const { api, store } = this.context;
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

    this.setState({ step: 'DEPLOYMENT' });

    const contract = api.newContract(abiParsed);

    deploy(contract, options, params, metadata, this.onDeploymentState)
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
        ])
        .then(() => {
          console.log(`contract deployed at ${address}`);
          this.setState({ step: 'COMPLETED', address });
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
        this.setState({
          deployState: (
            <FormattedMessage
              id='deployContract.state.preparing'
              defaultMessage='Preparing transaction for network transmission'
            />
          )
        });
        return;

      case 'checkRequest':
        this.setState({
          deployState: (
            <FormattedMessage
              id='deployContract.state.waitSigner'
              defaultMessage='Waiting for confirmation of the transaction in the Parity Secure Signer'
            />
          )
        });
        return;

      case 'getTransactionReceipt':
        this.setState({
          txhash: data.txhash,
          deployState: (
            <FormattedMessage
              id='deployContract.state.waitReceipt'
              defaultMessage='Waiting for the contract deployment transaction receipt'
            />
          )
        });
        return;

      case 'hasReceipt':
      case 'getCode':
        this.setState({
          deployState: (
            <FormattedMessage
              id='deployContract.state.validatingCode'
              defaultMessage='Validating the deployed contract code'
            />
          )
        });
        return;

      case 'confirmationNeeded':
        this.setState({
          deployState: (
            <FormattedMessage
              id='deployContract.state.confirmationNeeded'
              defaultMessage='The operation needs confirmations from the other owners of the contract'
            />
          )
        });
        return;

      case 'completed':
        this.setState({
          deployState: (
            <FormattedMessage
              id='deployContract.state.completed'
              defaultMessage='The contract deployment has been completed'
            />
          )
        });
        return;

      default:
        console.error('Unknown contract deployment state', data);
        return;
    }
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
