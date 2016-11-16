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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import CircularProgress from 'material-ui/CircularProgress';

import Contracts from '../../contracts';
import { Input, InputAddress } from '../Form';

import styles from './methodDecoding.css';

const ASCII_INPUT = /^[a-z0-9\s,?;.:/!()-_@'"#]+$/i;
const CONTRACT_CREATE = '0x60606040';
const TOKEN_METHODS = {
  '0xa9059cbb': 'transfer(to,value)'
};

class MethodDecoding extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    tokens: PropTypes.object,
    transaction: PropTypes.object,
    historic: PropTypes.bool
  }

  state = {
    contractAddress: null,
    method: null,
    methodName: null,
    methodInputs: null,
    methodParams: null,
    methodSignature: null,
    token: null,
    isContract: false,
    isDeploy: false,
    isReceived: false,
    isLoading: true
  }

  componentWillMount () {
    const lookupResult = this.lookup();

    if (typeof lookupResult === 'object' && typeof lookupResult.then === 'function') {
      lookupResult.then(() => this.setState({ isLoading: false }));
    } else {
      this.setState({ isLoading: false });
    }
  }

  render () {
    const { transaction } = this.props;
    const { isLoading } = this.state;

    if (!transaction) {
      return null;
    }

    if (isLoading) {
      return (
        <div className={ styles.loading }>
          <CircularProgress size={ 60 } thickness={ 2 } />
        </div>
      );
    }

    return (
      <div className={ styles.container }>
        { this.renderAction() }
        { this.renderGas() }
      </div>
    );
  }

  renderGas () {
    const { historic, transaction } = this.props;
    const { gas, gasPrice } = transaction;
    const gasValue = gas.mul(gasPrice);

    return (
      <div className={ styles.gasDetails }>
        <span>{ historic ? 'Provided' : 'Provides' } </span>
        <span className={ styles.highlight }>
          { gas.toFormat(0) } gas ({ gasPrice.div(1000000).toFormat(0) }M/<small>ETH</small>)
        </span>
        <span> for a total transaction value of </span>
        <span className={ styles.highlight }>{ this.renderEtherValue(gasValue) }</span>
      </div>
    );
  }

  renderAction () {
    const { methodName, methodInputs, methodSignature, token, isDeploy, isReceived, isContract } = this.state;

    if (isDeploy) {
      return this.renderDeploy();
    }

    if (isContract && methodSignature) {
      if (token && TOKEN_METHODS[methodSignature] && methodInputs) {
        return this.renderTokenAction();
      }

      if (methodName) {
        return this.renderSignatureMethod();
      }

      return this.renderUnknownMethod();
    }

    return isReceived
      ? this.renderValueReceipt()
      : this.renderValueTransfer();
  }

  renderInputValue () {
    const { api } = this.context;
    const { transaction } = this.props;

    if (!/^(0x)?([0]*[1-9a-f]+[0]*)+$/.test(transaction.input)) {
      return null;
    }

    const ascii = api.util.hex2Ascii(transaction.input);

    const text = ASCII_INPUT.test(ascii)
      ? ascii
      : transaction.input;

    return (
      <div>
        <span>with the input &nbsp;</span>
        <code className={ styles.inputData }>{ text }</code>
      </div>
    );
  }

  renderTokenAction () {
    const { historic } = this.props;
    const { methodSignature, methodInputs } = this.state;
    const [to, value] = methodInputs;
    const address = to.value;

    switch (TOKEN_METHODS[methodSignature]) {
      case 'transfer(to,value)':
      default:
        return (
          <div className={ styles.details }>
            <div>
              <span>{ historic ? 'Transferred' : 'Will transfer' } </span>
              <span className={ styles.highlight }>
                { this.renderTokenValue(value.value) }
              </span>
              <span> to </span>
            </div>

            { this.renderAddressName(address) }
          </div>
        );
    }
  }

  renderDeploy () {
    const { historic, transaction } = this.props;

    if (!historic) {
      return (
        <div className={ styles.details }>
          Will deploy a contract.
        </div>
      );
    }

    return (
      <div className={ styles.details }>
        <div>
          <span>Deployed a contract at address </span>
        </div>

        { this.renderAddressName(transaction.creates, false) }
      </div>
    );
  }

  renderValueReceipt () {
    const { historic, transaction } = this.props;
    const { isContract } = this.state;

    return (
      <div className={ styles.details }>
        <div>
          <span>{ historic ? 'Received' : 'Will receive' } </span>
          <span className={ styles.highlight }>
            { this.renderEtherValue(transaction.value) }
          </span>
          <span> from { isContract ? 'the contract' : '' } </span>
        </div>

        { this.renderAddressName(transaction.from) }
        { this.renderInputValue() }
      </div>
    );
  }

  renderValueTransfer () {
    const { historic, transaction } = this.props;
    const { isContract } = this.state;

    return (
      <div className={ styles.details }>
        <div>
          <span>{ historic ? 'Transferred' : 'Will transfer' } </span>
          <span className={ styles.highlight }>
            { this.renderEtherValue(transaction.value) }
          </span>
          <span> to { isContract ? 'the contract' : '' } </span>
        </div>

        { this.renderAddressName(transaction.to) }
        { this.renderInputValue() }
      </div>
    );
  }

  renderSignatureMethod () {
    const { historic, transaction } = this.props;
    const { methodName, methodInputs } = this.state;

    return (
      <div className={ styles.details }>
        <div className={ styles.description }>
          <div>
            <span>{ historic ? 'Executed' : 'Will execute' } the </span>
            <span className={ styles.name }>{ methodName }</span>
            <span> function on the contract </span>
          </div>

          { this.renderAddressName(transaction.to) }

          <div>
            <span>transferring </span>
            <span className={ styles.highlight }>
              { this.renderEtherValue(transaction.value) }
            </span>
            <span>
              { methodInputs.length ? ', passing the following parameters:' : '.' }
            </span>
          </div>
        </div>
        <div className={ styles.inputs }>
          { this.renderInputs() }
        </div>
      </div>
    );
  }

  renderUnknownMethod () {
    const { historic, transaction } = this.props;

    return (
      <div className={ styles.details }>
        <div>
          <span>{ historic ? 'Executed' : 'Will execute' } </span>
          <span className={ styles.name }>an unknown/unregistered</span>
          <span> method on the contract </span>
        </div>

        { this.renderAddressName(transaction.to) }

        <div>
          <span>transferring </span>
          <span className={ styles.highlight }>
            { this.renderEtherValue(transaction.value) }
          </span>
          <span>.</span>
        </div>
      </div>
    );
  }

  renderInputs () {
    const { methodInputs } = this.state;

    return methodInputs.map((input, index) => {
      switch (input.type) {
        case 'address':
          return (
            <InputAddress
              disabled
              text
              key={ index }
              className={ styles.input }
              value={ input.value }
              label={ input.type } />
          );

        default:
          return (
            <Input
              readOnly
              allowCopy
              key={ index }
              className={ styles.input }
              value={ this.renderValue(input.value) }
              label={ input.type } />
          );
      }
    });
  }

  renderValue (value) {
    const { api } = this.context;

    if (api.util.isInstanceOf(value, BigNumber)) {
      return value.toFormat(0);
    } else if (api.util.isArray(value)) {
      return api.util.bytesToHex(value);
    }

    return value.toString();
  }

  renderTokenValue (value) {
    const { token } = this.state;

    return (
      <span className={ styles.tokenValue }>
        { value.div(token.format).toFormat(5) }<small> { token.tag }</small>
      </span>
    );
  }

  renderEtherValue (value) {
    const { api } = this.context;
    const ether = api.util.fromWei(value);

    return (
      <span className={ styles.etherValue }>
        { ether.toFormat(5) }<small> ETH</small>
      </span>
    );
  }

  renderAddressName (address, withName = true) {
    return (
      <div className={ styles.addressContainer }>
        <InputAddress
          disabled
          className={ styles.address }
          value={ address }
          text={ withName }
        />
      </div>
    );
  }

  lookup () {
    const { transaction } = this.props;

    if (!transaction) {
      return;
    }

    const { api } = this.context;
    const { address, tokens } = this.props;

    const isReceived = transaction.to === address;
    const contractAddress = isReceived ? transaction.from : transaction.to;

    const token = (tokens || {})[contractAddress];
    this.setState({ token, isReceived, contractAddress });

    if (!transaction.input || transaction.input === '0x') {
      return;
    }

    if (contractAddress === '0x') {
      return;
    }

    return api.eth
      .getCode(contractAddress || transaction.creates)
      .then((bytecode) => {
        const isContract = bytecode && /^(0x)?([0]*[1-9a-f]+[0]*)+$/.test(bytecode);

        this.setState({ isContract });

        if (!isContract) {
          return;
        }

        const { signature, paramdata } = api.util.decodeCallData(transaction.input);
        this.setState({ methodSignature: signature, methodParams: paramdata });

        if (!signature || signature === CONTRACT_CREATE || transaction.creates) {
          this.setState({ isDeploy: true });
          return;
        }

        return Contracts.get()
          .signatureReg
          .lookup(signature)
          .then((method) => {
            let methodInputs = null;
            let methodName = null;

            if (method && method.length) {
              const { methodParams } = this.state;
              const abi = api.util.methodToAbi(method);

              methodName = abi.name;
              methodInputs = api.util
                .decodeMethodInput(abi, methodParams)
                .map((value, index) => {
                  const type = abi.inputs[index].type;

                  return { type, value };
                });
            }

            this.setState({
              method,
              methodName,
              methodInputs,
              bytecode
            });
          });
      })
      .catch((error) => {
        console.warn('lookup', error);
      });
  }
}

function mapStateToProps (state) {
  const { tokens } = state.balances;

  return { tokens };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(MethodDecoding);
