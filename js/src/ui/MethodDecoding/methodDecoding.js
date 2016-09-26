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

import { Input, InputAddress } from '../Form';

import styles from './methodDecoding.css';

const CONTRACT_CREATE = '0x60606040';

export default class Method extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    contracts: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    tokens: PropTypes.object,
    transaction: PropTypes.object,
    historic: PropTypes.bool,
    isContract: PropTypes.bool,
    isReceived: PropTypes.bool
  }

  state = {
    signature: null,
    paramdata: null,
    name: null,
    inputs: null
  }

  componentDidMount () {
    const { transaction } = this.props;

    this.lookup(transaction.input);
  }

  componentWillReceiveProps (newProps) {
    const { transaction } = this.props;

    if (newProps.transaction === transaction) {
      return;
    }

    this.lookup(transaction.input);
  }

  render () {
    const { isReceived } = this.props;
    const { name } = this.state;

    if (!name) {
      return isReceived
        ? this.renderValueReceipt()
        : this.renderValueTransfer();
    }

    return this.renderUnknown();
  }

  renderValueReceipt () {
    const { historic, transaction, isContract } = this.props;

    return (
      <div className={ styles.details }>
        { historic ? 'Received' : 'Will receive' } <span className={ styles.highlight }>{ this.renderEther(transaction.value) }</span> { isContract ? 'from the contract.' : 'from the sender.' }
      </div>
    );
  }

  renderValueTransfer () {
    const { historic, transaction, isContract } = this.props;

    return (
      <div className={ styles.details }>
        { historic ? 'Transferred' : 'Will transfer' } a value of <span className={ styles.highlight }>{ this.renderEther(transaction.value) }</span> { isContract ? 'to the contract.' : 'to the recipient.' }
      </div>
    );
  }

  renderSimple () {
    const { name } = this.state;

    return (
      <div className={ styles.details }>
        <div className={ styles.name }>
          { name }
        </div>
        <div className={ styles.inputs }>
          { this.renderInputs() }
        </div>
      </div>
    );
  }

  renderUnknown () {
    const { historic } = this.props;
    const { name } = this.state;

    return (
      <div className={ styles.details }>
        <div className={ styles.description }>
          { historic ? 'Executed' : 'Will execute' } the <span className={ styles.name }>{ name }</span> function on the contract, passing the following values as part of the transaction:
        </div>
        <div className={ styles.inputs }>
          { this.renderInputs() }
        </div>
      </div>
    );
  }

  renderInputs () {
    const { accounts, contacts, tokens } = this.props;
    const { inputs } = this.state;

    return inputs.map((input, index) => {
      switch (input.type) {
        case 'address':
          const address = input.value;
          const account = (accounts || {})[address] || (contacts || {})[address] || (tokens || {})[address];
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

  renderEther (value) {
    const { api } = this.context;
    const ether = api.util.fromWei(value);

    return (
      <span className={ styles.ether }>
        { ether.toFormat(5) }<small>ΞTH</small>
      </span>
    );
  }

  lookup (input) {
    const { api, contracts } = this.context;

    if (!input) {
      return;
    }

    const { signature, paramdata } = api.util.decodeCallData(input);

    if (!signature || signature === CONTRACT_CREATE) {
      return;
    }

    this.setState({ signature, paramdata });

    contracts.signatureReg
      .lookup(signature)
      .then(([method, owner]) => {
        let inputs = null;
        let name = null;

        if (method && method.length) {
          const abi = api.util.methodToAbi(method);

          name = abi.name;
          inputs = api.util
            .decodeMethodInput(abi, paramdata)
            .map((value, index) => {
              const type = abi.inputs[index].type;

              return { type, value };
            });
        }

        this.setState({ method, name, inputs });
      })
      .catch((error) => {
        console.error('lookup', error);
      });
  }
}
