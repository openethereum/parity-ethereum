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
import { bindActionCreators } from 'redux';
import { observer } from 'mobx-react';
import { pick } from 'lodash';

import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowBack from 'material-ui/svg-icons/navigation/arrow-back';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import { toWei } from '~/api/util/wei';
import { BusyStep, Button, CompletedStep, GasPriceEditor, IdentityIcon, Modal, TxHash } from '~/ui';
import { MAX_GAS_ESTIMATION } from '~/util/constants';
import { validateAddress, validateUint } from '~/util/validation';
import { parseAbiType } from '~/util/abi';

import DetailsStep from './DetailsStep';

import { ERROR_CODES } from '~/api/transport/error';

const STEP_DETAILS = 0;
const STEP_BUSY_OR_GAS = 1;
const STEP_BUSY = 2;

const TITLES = {
  transfer: 'function details',
  sending: 'sending',
  complete: 'complete',
  gas: 'gas selection',
  rejected: 'rejected'
};
const STAGES_BASIC = [TITLES.transfer, TITLES.sending, TITLES.complete];
const STAGES_GAS = [TITLES.transfer, TITLES.gas, TITLES.sending, TITLES.complete];

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
    isTest: PropTypes.bool,
    onClose: PropTypes.func.isRequired,
    onFromAddressChange: PropTypes.func.isRequired
  }

  gasStore = new GasPriceEditor.Store(this.context.api, { gasLimit: this.props.gasLimit });

  state = {
    amount: '0',
    amountError: null,
    busyState: null,
    fromAddressError: null,
    func: null,
    funcError: null,
    gasEdit: false,
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
    const { sending, step, gasEdit, rejected } = this.state;
    const steps = gasEdit ? STAGES_GAS : STAGES_BASIC;

    if (rejected) {
      steps[steps.length - 1] = TITLES.rejected;
    }

    return (
      <Modal
        actions={ this.renderDialogActions() }
        busy={ sending }
        current={ step }
        steps={ steps }
        visible
        waiting={ gasEdit ? [STEP_BUSY] : [STEP_BUSY_OR_GAS] }>
        { this.renderStep() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { onClose, fromAddress } = this.props;
    const { gasEdit, sending, step, fromAddressError, valuesError } = this.state;
    const hasError = fromAddressError || valuesError.find((error) => error);

    const cancelBtn = (
      <Button
        key='cancel'
        label='Cancel'
        icon={ <ContentClear /> }
        onClick={ onClose } />
    );
    const postBtn = (
      <Button
        key='postTransaction'
        label='post transaction'
        disabled={ !!(sending || hasError) }
        icon={ <IdentityIcon address={ fromAddress } button /> }
        onClick={ this.postTransaction } />
    );
    const nextBtn = (
      <Button
        key='nextStep'
        label='next'
        icon={ <NavigationArrowForward /> }
        onClick={ this.onNextClick } />
    );
    const prevBtn = (
      <Button
        key='prevStep'
        label='prev'
        icon={ <NavigationArrowBack /> }
        onClick={ this.onPrevClick } />
    );

    if (step === STEP_DETAILS) {
      return [
        cancelBtn,
        gasEdit ? nextBtn : postBtn
      ];
    } else if (step === (gasEdit ? STEP_BUSY : STEP_BUSY_OR_GAS)) {
      return [
        cancelBtn
      ];
    } else if (gasEdit && (step === STEP_BUSY_OR_GAS)) {
      return [
        cancelBtn,
        prevBtn,
        postBtn
      ];
    }

    return [
      <Button
        key='close'
        label='Done'
        icon={ <ActionDoneAll /> }
        onClick={ onClose } />
    ];
  }

  renderStep () {
    const { onFromAddressChange } = this.props;
    const { gasEdit, step, busyState, txhash, rejected } = this.state;
    const { errorEstimated } = this.gasStore;

    if (rejected) {
      return (
        <BusyStep
          title='The execution has been rejected'
          state='You can safely close this window, the function execution will not occur.'
        />
      );
    }

    if (step === STEP_DETAILS) {
      return (
        <DetailsStep
          { ...this.props }
          { ...this.state }
          warning={ errorEstimated }
          onAmountChange={ this.onAmountChange }
          onFromAddressChange={ onFromAddressChange }
          onFuncChange={ this.onFuncChange }
          onGasEditClick={ this.onGasEditClick }
          onValueChange={ this.onValueChange } />
      );
    } else if (step === (gasEdit ? STEP_BUSY : STEP_BUSY_OR_GAS)) {
      return (
        <BusyStep
          title='The function execution is in progress'
          state={ busyState } />
      );
    } else if (gasEdit && (step === STEP_BUSY_OR_GAS)) {
      return (
        <GasPriceEditor
          store={ this.gasStore } />
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
        console.warn('estimateGas', error);
      });
  }

  postTransaction = () => {
    const { api, store } = this.context;
    const { fromAddress } = this.props;
    const { amount, func, gasEdit, values } = this.state;
    const steps = gasEdit ? STAGES_GAS : STAGES_BASIC;
    const finalstep = steps.length - 1;
    const options = {
      gas: this.gasStore.gas,
      gasPrice: this.gasStore.price,
      from: fromAddress,
      value: api.util.toWei(amount || 0)
    };

    this.setState({ sending: true, step: gasEdit ? STEP_BUSY : STEP_BUSY_OR_GAS });

    func
      .postTransaction(options, values)
      .then((requestId) => {
        this.setState({ busyState: 'Waiting for authorization in the Parity Signer' });

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
        this.setState({ sending: false, step: finalstep, txhash, busyState: 'Your transaction has been posted to the network' });
      })
      .catch((error) => {
        console.error('postTransaction', error);
        store.dispatch({ type: 'newError', error });
      });
  }

  onGasEditClick = () => {
    this.setState({
      gasEdit: !this.state.gasEdit
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
}

function mapStateToProps (initState, initProps) {
  const fromAddresses = Object.keys(initProps.accounts);

  return (state) => {
    const balances = pick(state.balances.balances, fromAddresses);
    const { gasLimit } = state.nodeStatus;

    return { gasLimit, balances };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(ExecuteContract);
