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

import { CircularProgress } from 'material-ui';
import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import { TypedInput, InputAddress } from '../Form';
import MethodDecodingStore from './methodDecodingStore';

import styles from './methodDecoding.css';

const ASCII_INPUT = /^[a-z0-9\s,?;.:/!()-_@'"#]+$/i;
const TOKEN_METHODS = {
  '0xa9059cbb': 'transfer(to,value)'
};

class MethodDecoding extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    token: PropTypes.object,
    transaction: PropTypes.object,
    historic: PropTypes.bool
  }

  state = {
    contractAddress: null,
    methodName: null,
    methodInputs: null,
    methodParams: null,
    methodSignature: null,
    isContract: false,
    isDeploy: false,
    isReceived: false,
    isLoading: true,
    expandInput: false,
    inputType: 'auto'
  }

  methodDecodingStore = MethodDecodingStore.get(this.context.api);

  componentWillMount () {
    const { address, transaction } = this.props;

    this
      .methodDecodingStore
      .lookup(address, transaction)
      .then((lookup) => {
        const newState = {
          methodName: lookup.name,
          methodInputs: lookup.inputs,
          methodParams: lookup.params,
          methodSignature: lookup.signature,

          isContract: lookup.contract,
          isDeploy: lookup.deploy,
          isLoading: false,
          isReceived: lookup.received
        };

        this.setState(newState);
      });
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

    if (!gas || !gasPrice) {
      return null;
    }

    const gasValue = gas.mul(gasPrice);

    return (
      <div className={ styles.gasDetails }>
        <span>{ historic ? 'Provided' : 'Provides' } </span>
        <span className={ styles.highlight }>
          { gas.toFormat(0) } gas ({ gasPrice.div(1000000).toFormat(0) }M/<small>ETH</small>)
        </span>
        {
          transaction.gasUsed
            ? (
              <span>
                <span>used</span>
                <span className={ styles.highlight }>
                  { transaction.gasUsed.toFormat(0) } gas
                </span>
              </span>
            )
            : null
        }
        <span> for a total transaction value of </span>
        <span className={ styles.highlight }>{ this.renderEtherValue(gasValue) }</span>
        { this.renderMinBlock() }
      </div>
    );
  }

  renderMinBlock () {
    const { historic, transaction } = this.props;
    const { condition } = transaction;

    if (!condition) {
      return null;
    }

    if (condition.block && condition.block.gt(0)) {
      return (
        <span>, { historic ? 'Submitted' : 'Submission' } at block <span className={ styles.highlight }>#{ condition.block.toFormat(0) }</span></span>
      );
    }

    if (condition.time) {
      return (
        <span>, { historic ? 'Submitted' : 'Submission' } at <span className={ styles.highlight }>{ moment(condition.time).format('LLLL') }</span></span>
      );
    }

    return null;
  }

  renderAction () {
    const { token } = this.props;
    const { methodName, methodInputs, methodSignature, isDeploy, isReceived, isContract } = this.state;

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

  getAscii () {
    const { api } = this.context;
    const { transaction } = this.props;
    const ascii = api.util.hexToAscii(transaction.input || transaction.data);

    return { value: ascii, valid: ASCII_INPUT.test(ascii) };
  }

  renderInputValue () {
    const { transaction } = this.props;
    const { expandInput, inputType } = this.state;
    const input = transaction.input || transaction.data;

    if (!/^(0x)?([0]*[1-9a-f]+[0]*)+$/.test(input)) {
      return null;
    }

    const ascii = this.getAscii();
    const type = inputType === 'auto'
      ? (ascii.valid ? 'ascii' : 'raw')
      : inputType;

    const text = type === 'ascii'
      ? ascii.value
      : input;

    const expandable = text.length > 50;
    const textToShow = expandInput || !expandable
      ? text
      : text.slice(0, 50) + '...';

    return (
      <div className={ styles.details }>
        <span>with the </span>
        <span
          onClick={ this.toggleInputType }
          className={ [ styles.clickable, styles.noSelect ].join(' ') }
        >
          { type === 'ascii' ? 'input' : 'data' }
        </span>
        <span> &nbsp; </span>
        <span
          onClick={ this.toggleInputExpand }
          className={ expandable ? styles.clickable : '' }
        >
          <code className={ styles.inputData }>
            { textToShow }
          </code>
        </span>
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
    const { methodInputs } = this.state;

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

        <div>
          { methodInputs && methodInputs.length ? 'with the following parameters:' : ''}
        </div>

        <div className={ styles.inputs }>
          { this.renderInputs() }
        </div>
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

    if (!methodInputs || methodInputs.length === 0) {
      return null;
    }

    const inputs = methodInputs.map((input, index) => {
      const label = input.name
        ? `${input.name}: ${input.type}`
        : input.type;

      return (
        <TypedInput
          allowCopy
          className={ styles.input }
          label={ label }
          key={ index }
          param={ input.type }
          readOnly
          value={ this.renderValue(input.value) }
        />
      );
    });

    return inputs;
  }

  renderValue (value) {
    const { api } = this.context;

    if (api.util.isArray(value)) {
      return api.util.bytesToHex(value);
    }

    return value.toString();
  }

  renderTokenValue (value) {
    const { token } = this.props;

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

  toggleInputExpand = () => {
    if (window.getSelection && window.getSelection().type === 'Range') {
      return;
    }

    this.setState({
      expandInput: !this.state.expandInput
    });
  }

  toggleInputType = () => {
    const { inputType } = this.state;

    if (inputType !== 'auto') {
      return this.setState({
        inputType: this.state.inputType === 'raw' ? 'ascii' : 'raw'
      });
    }

    const ascii = this.getAscii();

    return this.setState({
      inputType: ascii.valid ? 'raw' : 'ascii'
    });
  }
}

function mapStateToProps (initState, initProps) {
  const { tokens } = initState.balances;
  const { address } = initProps;

  const token = (tokens || {})[address];

  return () => {
    return { token };
  };
}

export default connect(
  mapStateToProps,
  null
)(MethodDecoding);
