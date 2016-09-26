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

import Contracts from '../../contracts';
import { Input, InputAddress } from '../Form';

import styles from './methodDecoding.css';

const CONTRACT_CREATE = '0x60606040';
const TOKEN_METHODS = {
  '0xa9059cbb': 'transfer(to,value)'
};

export default class Method extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    tokens: PropTypes.object,
    transaction: PropTypes.object,
    historic: PropTypes.bool
  }

  state = {
    method: null,
    methodName: null,
    methodInputs: null,
    methodParams: null,
    methodSignature: null,
    token: null,
    isContract: false,
    isDeploy: false,
    isReceived: false
  }

  componentDidMount () {
    const { transaction } = this.props;

    this.lookup(transaction);
  }

  componentWillReceiveProps (newProps) {
    const { transaction } = this.props;

    if (newProps.transaction === transaction) {
      return;
    }

    this.lookup(transaction);
  }

  render () {
    const { methodName, methodInputs, methodSignature, token, isDeploy, isReceived } = this.state;

    if (isDeploy) {
      return this.renderDeploy();
    }

    if (methodSignature) {
      if (token && TOKEN_METHODS[methodSignature] && methodInputs) {
        return this.renderTokenAction();
      }

      return methodName
        ? this.renderSignatureMethod()
        : this.renderUnknownMethod();
    }

    return isReceived
      ? this.renderValueReceipt()
      : this.renderValueTransfer();
  }

  renderTokenAction () {
    const { historic } = this.props;
    const { methodSignature, methodInputs } = this.state;
    const [to, value] = methodInputs;
    const address = to.value;
    const account = this.getAccount(address);
    const name = account ? account.name.toUpperCase() : null;

    switch (TOKEN_METHODS[methodSignature]) {
      case 'transfer(to,value)':
      default:
        return (
          <div className={ styles.details }>
            { historic ? 'Transferred' : 'Will transfer' } <span className={ styles.highlight }>{ this.renderTokenValue(value.value) }</span> to <span className={ styles.highlight }>{ name || address }</span>.
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
        Deployed a contract at address <span className={ styles.highlight }>{ transaction.creates }</span>.
      </div>
    );
  }

  renderValueReceipt () {
    const { historic, transaction } = this.props;
    const { isContract } = this.state;

    return (
      <div className={ styles.details }>
        { historic ? 'Received' : 'Will receive' } <span className={ styles.highlight }>{ this.renderEtherValue(transaction.value) }</span> { isContract ? 'from the contract.' : 'from the sender.' }
      </div>
    );
  }

  renderValueTransfer () {
    const { historic, transaction } = this.props;
    const { isContract } = this.state;

    return (
      <div className={ styles.details }>
        { historic ? 'Transferred' : 'Will transfer' } a value of <span className={ styles.highlight }>{ this.renderEtherValue(transaction.value) }</span> { isContract ? 'to the contract.' : 'to the recipient.' }
      </div>
    );
  }

  renderSimple () {
    const { methodName } = this.state;

    return (
      <div className={ styles.details }>
        <div className={ styles.name }>
          { methodName }
        </div>
        <div className={ styles.inputs }>
          { this.renderInputs() }
        </div>
      </div>
    );
  }

  renderSignatureMethod () {
    const { historic, transaction } = this.props;
    const { methodName } = this.state;

    return (
      <div className={ styles.details }>
        <div className={ styles.description }>
          { historic ? 'Executed' : 'Will execute' } the <span className={ styles.name }>{ methodName }</span> function on the contract, transferring <span className={ styles.highlight }>{ this.renderEtherValue(transaction.value) }</span> and passing the following values as part of the transaction:
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
        { historic ? 'Executed' : 'Will execute' } <span className={ styles.name }>an unknown/unregistered</span> method on the contract, transferring <span className={ styles.highlight }>{ this.renderEtherValue(transaction.value) }</span>.
      </div>
    );
  }

  renderInputs () {
    const { methodInputs } = this.state;

    return methodInputs.map((input, index) => {
      switch (input.type) {
        case 'address':
          const address = input.value;
          const account = this.getAccount(address);
          const name = account ? account.name.toUpperCase() : null;

          return (
            <InputAddress
              disabled
              key={ index }
              className={ styles.input }
              value={ input.value }
              nameValue={ name }
              label={ input.type } />
          );

        default:
          return (
            <Input
              disabled
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
        { value.div(token.format).toFormat(5) }<small>{ token.tag }</small>
      </span>
    );
  }

  renderEtherValue (value) {
    const { api } = this.context;
    const ether = api.util.fromWei(value);

    return (
      <span className={ styles.etherValue }>
        { ether.toFormat(5) }<small>ΞTH</small>
      </span>
    );
  }

  getAccount (address) {
    const { accounts, contacts, contracts, tokens } = this.props;

    return (accounts || {})[address] ||
      (contacts || {})[address] ||
      (tokens || {})[address] ||
      (contracts || {})[address];
  }

  lookup (transaction) {
    const { api } = this.context;
    const { address, tokens } = this.props;

    if (!transaction || !transaction.input) {
      return;
    }

    const isReceived = transaction.to === address;
    const token = (tokens || {})[isReceived ? transaction.from : transaction.to];
    this.setState({ token, isReceived });

    const { signature, paramdata } = api.util.decodeCallData(transaction.input);
    this.setState({ methodSignature: signature, methodParams: paramdata });

    if (!signature || signature === CONTRACT_CREATE) {
      this.setState({ isDeploy: true });
      return;
    }

    api.eth
      .getCode(isReceived ? transaction.from : transaction.to)
      .then((code) => {
        if (code && code !== '0x') {
          this.setState({ isContract: true });
        }

        return Contracts.get().signatureReg.lookup(signature);
      }).then((method) => {
        let methodInputs = null;
        let methodName = null;

        if (method && method.length) {
          const abi = api.util.methodToAbi(method);

          methodName = abi.name;
          methodInputs = api.util
            .decodeMethodInput(abi, paramdata)
            .map((value, index) => {
              const type = abi.inputs[index].type;

              return { type, value };
            });
        }

        this.setState({ method, methodName, methodInputs });
      })
      .catch((error) => {
        console.error('lookup', error);
      });
  }
}
