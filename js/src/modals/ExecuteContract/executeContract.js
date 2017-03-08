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

import { toWei } from '~/api/util/wei';
import { BusyStep, Button, CompletedStep, GasPriceEditor, IdentityIcon, Portal, TxHash, Warning } from '~/ui';
import { CancelIcon, DoneIcon, NextIcon, PrevIcon } from '~/ui/Icons';
import { MAX_GAS_ESTIMATION } from '~/util/constants';
import { validateAddress, validateUint } from '~/util/validation';
import { parseAbiType } from '~/util/abi';

import AdvancedStep from './AdvancedStep';
import DetailsStep from './DetailsStep';

import { ERROR_CODES } from '~/api/transport/error';

const STEP_DETAILS = 0;
const STEP_BUSY_OR_ADVANCED = 1;
const STEP_BUSY = 2;

const TITLES = {
  transfer: (
    <FormattedMessage
      id='executeContract.steps.transfer'
      defaultMessage='function details'
    />
  ),
  sending: (
    <FormattedMessage
      id='executeContract.steps.sending'
      defaultMessage='sending'
    />
  ),
  complete: (
    <FormattedMessage
      id='executeContract.steps.complete'
      defaultMessage='complete'
    />
  ),
  advanced: (
    <FormattedMessage
      id='executeContract.steps.advanced'
      defaultMessage='advanced options'
    />
  ),
  rejected: (
    <FormattedMessage
      id='executeContract.steps.rejected'
      defaultMessage='rejected'
    />
  )
};
const STAGES_BASIC = [TITLES.transfer, TITLES.sending, TITLES.complete];
const STAGES_ADVANCED = [TITLES.transfer, TITLES.advanced, TITLES.sending, TITLES.complete];

@observer
class ExecuteContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object,
    balances: PropTypes.object,
    contract: PropTypes.object.isRequired,
    fromAddress: PropTypes.string,
    gasLimit: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired
  }

  gasStore = new GasPriceEditor.Store(this.context.api, { gasLimit: this.props.gasLimit });

  state = {
    advancedOptions: false,
    amount: '0',
    amountError: null,
    busyState: null,
    fromAddressError: null,
    func: null,
    funcError: null,
    rejected: false,
    sending: false,
    step: STEP_DETAILS,
    txhash: null,
    values: [],
    valuesError: []
  }

  componentDidMount () {
    const { contract } = this.props;
    const functions = contract.functions
      .filter((func) => !func.constant)
      .sort((a, b) => (a.name || '').localeCompare(b.name || ''));

    this.onFuncChange(null, functions[0]);
  }

  componentWillReceiveProps (newProps) {
    if (newProps.fromAddress !== this.props.fromAddress) {
      this.estimateGas(newProps.fromAddress);
    }
  }

  render () {
    const { advancedOptions, rejected, sending, step } = this.state;
    const steps = advancedOptions ? STAGES_ADVANCED : STAGES_BASIC;

    if (rejected) {
      steps[steps.length - 1] = TITLES.rejected;
    }

    return (
      <Portal
        activeStep={ step }
        buttons={ this.renderDialogActions() }
        busySteps={
          advancedOptions
            ? [STEP_BUSY]
            : [STEP_BUSY_OR_ADVANCED]
        }
        busy={ sending }
        onClose={ this.onClose }
        open
        steps={ steps }
      >
        { this.renderExceptionWarning() }
        { this.renderStep() }
      </Portal>
    );
  }

  renderExceptionWarning () {
    const { gasEdit, step } = this.state;
    const { errorEstimated } = this.gasStore;

    if (!errorEstimated || step >= (gasEdit ? STEP_BUSY : STEP_BUSY_OR_ADVANCED)) {
      return null;
    }

    return (
      <Warning warning={ errorEstimated } />
    );
  }

  renderDialogActions () {
    const { fromAddress } = this.props;
    const { advancedOptions, sending, step, fromAddressError, valuesError } = this.state;
    const hasError = fromAddressError || valuesError.find((error) => error);

    const cancelBtn = (
      <Button
        key='cancel'
        label={
          <FormattedMessage
            id='executeContract.button.cancel'
            defaultMessage='cancel'
          />
        }
        icon={ <CancelIcon /> }
        onClick={ this.onClose }
      />
    );
    const postBtn = (
      <Button
        key='postTransaction'
        label={
          <FormattedMessage
            id='executeContract.button.post'
            defaultMessage='post transaction'
          />
        }
        disabled={ !!(sending || hasError) }
        icon={ <IdentityIcon address={ fromAddress } button /> }
        onClick={ this.postTransaction }
      />
    );
    const nextBtn = (
      <Button
        key='nextStep'
        label={
          <FormattedMessage
            id='executeContract.button.next'
            defaultMessage='next'
          />
        }
        icon={ <NextIcon /> }
        onClick={ this.onNextClick }
      />
    );
    const prevBtn = (
      <Button
        key='prevStep'
        label={
          <FormattedMessage
            id='executeContract.button.prev'
            defaultMessage='prev'
          />
        }
        icon={ <PrevIcon /> }
        onClick={ this.onPrevClick }
      />
    );

    if (step === STEP_DETAILS) {
      return [
        cancelBtn,
        advancedOptions ? nextBtn : postBtn
      ];
    } else if (step === (advancedOptions ? STEP_BUSY : STEP_BUSY_OR_ADVANCED)) {
      return [
        cancelBtn
      ];
    } else if (advancedOptions && (step === STEP_BUSY_OR_ADVANCED)) {
      return [
        cancelBtn,
        prevBtn,
        postBtn
      ];
    }

    return [
      <Button
        key='close'
        label={
          <FormattedMessage
            id='executeContract.button.done'
            defaultMessage='done'
          />
        }
        icon={ <DoneIcon /> }
        onClick={ this.onClose }
      />
    ];
  }

  renderStep () {
    const { onFromAddressChange } = this.props;
    const { advancedOptions, step, busyState, txhash, rejected } = this.state;

    if (rejected) {
      return (
        <BusyStep
          title={
            <FormattedMessage
              id='executeContract.rejected.title'
              defaultMessage='The execution has been rejected'
            />
          }
          state={
            <FormattedMessage
              id='executeContract.rejected.state'
              defaultMessage='You can safely close this window, the function execution will not occur.'
            />
          }
        />
      );
    }

    if (step === STEP_DETAILS) {
      return (
        <DetailsStep
          { ...this.props }
          { ...this.state }
          onAmountChange={ this.onAmountChange }
          onFromAddressChange={ onFromAddressChange }
          onFuncChange={ this.onFuncChange }
          onAdvancedClick={ this.onAdvancedClick }
          onValueChange={ this.onValueChange }
        />
      );
    } else if (step === (advancedOptions ? STEP_BUSY : STEP_BUSY_OR_ADVANCED)) {
      return (
        <BusyStep
          title={
            <FormattedMessage
              id='executeContract.busy.title'
              defaultMessage='The function execution is in progress'
            />
          }
          state={ busyState }
        />
      );
    } else if (advancedOptions && (step === STEP_BUSY_OR_ADVANCED)) {
      return (
        <AdvancedStep gasStore={ this.gasStore } />
      );
    }

    return (
      <CompletedStep>
        <TxHash hash={ txhash } />
      </CompletedStep>
    );
  }

  onAmountChange = (amount) => {
    this.gasStore.setEthValue(amount);
    this.setState({ amount }, this.estimateGas);
  }

  onFuncChange = (event, func) => {
    const values = (func.abi.inputs || []).map((input) => {
      const parsedType = parseAbiType(input.type);

      return parsedType.default;
    });

    this.setState({
      func,
      values
    }, this.estimateGas);
  }

  onValueChange = (event, index, _value) => {
    const { func, values, valuesError } = this.state;
    const input = func.inputs.find((input, _index) => index === _index);
    let value = _value;
    let valueError = null;

    switch (input.kind.type) {
      case 'address':
        valueError = validateAddress(_value).addressError;
        break;

      case 'uint':
        valueError = validateUint(_value).valueError;
        break;
    }

    values[index] = value;
    valuesError[index] = valueError;

    this.setState({
      values: [].concat(values),
      valuesError: [].concat(valuesError)
    }, () => {
      if (!valueError) {
        this.estimateGas();
      }
    });
  }

  estimateGas = (_fromAddress) => {
    const { fromAddress } = this.props;
    const { amount, func, values } = this.state;
    const options = {
      gas: MAX_GAS_ESTIMATION,
      from: _fromAddress || fromAddress,
      value: toWei(amount || 0)
    };

    if (!func) {
      return;
    }

    func
      .estimateGas(options, values)
      .then((gasEst) => {
        const gas = gasEst.mul(1.2);

        console.log(`estimateGas: received ${gasEst.toFormat(0)}, adjusted to ${gas.toFormat(0)}`);

        this.gasStore.setEstimated(gasEst.toFixed(0));
        this.gasStore.setGas(gas.toFixed(0));
      })
      .catch((error) => {
        this.gasStore.setEstimatedError();
        console.warn('estimateGas', error);
      });
  }

  postTransaction = () => {
    const { api, store } = this.context;
    const { fromAddress } = this.props;
    const { advancedOptions, amount, func, values } = this.state;
    const steps = advancedOptions ? STAGES_ADVANCED : STAGES_BASIC;
    const finalstep = steps.length - 1;

    const options = this.gasStore.overrideTransaction({
      from: fromAddress,
      value: api.util.toWei(amount || 0)
    });

    this.setState({ sending: true, step: advancedOptions ? STEP_BUSY : STEP_BUSY_OR_ADVANCED });

    func
      .postTransaction(options, values)
      .then((requestId) => {
        this.setState({
          busyState: (
            <FormattedMessage
              id='executeContract.busy.waitAuth'
              defaultMessage='Waiting for authorization in the Parity Signer'
            />
          )
        });

        return api
          .pollMethod('parity_checkRequest', requestId)
          .catch((error) => {
            if (error.code === ERROR_CODES.REQUEST_REJECTED) {
              this.setState({ rejected: true, step: finalstep });
              return false;
            }

            throw error;
          });
      })
      .then((txhash) => {
        this.setState({
          sending: false,
          step: finalstep,
          txhash,
          busyState: (
            <FormattedMessage
              id='executeContract.busy.posted'
              defaultMessage='Your transaction has been posted to the network'
            />
          )
        });
      })
      .catch((error) => {
        console.error('postTransaction', error);
        store.dispatch({ type: 'newError', error });
      });
  }

  onAdvancedClick = () => {
    this.setState({
      advancedOptions: !this.state.advancedOptions
    });
  }

  onNextClick = () => {
    this.setState({
      step: this.state.step + 1
    });
  }

  onPrevClick = () => {
    this.setState({
      step: this.state.step - 1
    });
  }

  onClose = () => {
    this.props.onClose();
  }
}

function mapStateToProps (initState, initProps) {
  const fromAddresses = Object.keys(initProps.accounts);

  return (state) => {
    const balances = pick(state.balances.balances, fromAddresses);
    const { gasLimit } = state.nodeStatus;

    return { gasLimit, balances };
  };
}

export default connect(
  mapStateToProps,
  null
)(ExecuteContract);
