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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { toWei } from '~/api/util/wei';
import { Button, GasPriceEditor, IdentityIcon, Portal, Warning } from '~/ui';
import { CancelIcon, NextIcon, PrevIcon } from '~/ui/Icons';
import { MAX_GAS_ESTIMATION } from '~/util/constants';
import { validateAddress, validateUint } from '~/util/validation';
import { parseAbiType } from '~/util/abi';
import { setSender } from '~/util/tx';

import AdvancedStep from './AdvancedStep';
import DetailsStep from './DetailsStep';

const STEP_DETAILS = 0;

const TITLES = {
  transfer: (
    <FormattedMessage
      id='executeContract.steps.transfer'
      defaultMessage='function details'
    />
  ),
  advanced: (
    <FormattedMessage
      id='executeContract.steps.advanced'
      defaultMessage='advanced options'
    />
  )
};
const STAGES_BASIC = [TITLES.transfer];
const STAGES_ADVANCED = [TITLES.transfer, TITLES.advanced];

@observer
class ExecuteContract extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object,
    contract: PropTypes.object.isRequired,
    fromAddress: PropTypes.string,
    gasLimit: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired
  };

  gasStore = new GasPriceEditor.Store(this.context.api, { gasLimit: this.props.gasLimit });

  state = {
    advancedOptions: false,
    amount: '0',
    amountError: null,
    fromAddressError: null,
    func: null,
    funcError: null,
    step: STEP_DETAILS,
    values: [],
    valuesError: []
  };

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
    const { advancedOptions, step } = this.state;
    const steps = advancedOptions ? STAGES_ADVANCED : STAGES_BASIC;

    return (
      <Portal
        activeStep={ step }
        buttons={ this.renderDialogActions() }
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
    const { errorEstimated } = this.gasStore;

    if (!errorEstimated) {
      return null;
    }

    return (
      <Warning warning={ errorEstimated } />
    );
  }

  renderDialogActions () {
    const { fromAddress } = this.props;
    const { advancedOptions, step, fromAddressError, valuesError } = this.state;
    const hasError = !!(fromAddressError || valuesError.find((error) => error));

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
        disabled={ hasError }
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
    }

    return [
      cancelBtn,
      prevBtn,
      postBtn
    ];
  }

  renderStep () {
    const { accounts, contract, fromAddress, onFromAddressChange } = this.props;
    const { step } = this.state;

    if (step === STEP_DETAILS) {
      return (
        <DetailsStep
          { ...this.state }
          accounts={ accounts }
          contract={ contract }
          fromAddress={ fromAddress }
          onAmountChange={ this.onAmountChange }
          onFromAddressChange={ onFromAddressChange }
          onFuncChange={ this.onFuncChange }
          onAdvancedClick={ this.onAdvancedClick }
          onValueChange={ this.onValueChange }
        />
      );
    }

    return (
      <AdvancedStep gasStore={ this.gasStore } />
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
    const { api } = this.context;
    const { fromAddress } = this.props;
    const { amount, func, values } = this.state;

    const options = this.gasStore.overrideTransaction({
      from: fromAddress,
      value: api.util.toWei(amount || 0)
    });

    setSender(fromAddress);
    func.postTransaction(options, values);
    this.onClose();
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

function mapStateToProps (state) {
  const { gasLimit } = state.nodeStatus;

  return { gasLimit };
}

export default connect(
  mapStateToProps,
  null
)(ExecuteContract);
