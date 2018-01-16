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
import { CircularProgress } from 'material-ui';
import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
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
  };

  static propTypes = {
    address: PropTypes.string,
    compact: PropTypes.bool,
    token: PropTypes.object,
    transaction: PropTypes.object,
    historic: PropTypes.bool
  };

  static defaultProps = {
    address: '',
    compact: false,
    historic: false
  };

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
  };

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
    const { compact, historic, transaction } = this.props;
    const { gas, gasPrice, value } = transaction;

    if (!gas || !gasPrice || compact) {
      return null;
    }

    const gasProvided = (
      <span className={ styles.highlight }>
        <FormattedMessage
          id='ui.methodDecoding.gasValues'
          defaultMessage='{gas} gas ({gasPrice}M/{tag})'
          values={ {
            gas: gas.toFormat(0),
            gasPrice: gasPrice.div(1000000).toFormat(0),
            tag: <small>ETH</small>
          } }
        />
      </span>
    );
    const totalEthValue = (
      <span className={ styles.highlight }>
        { this.renderEtherValue(gas.mul(gasPrice).plus(value || 0)) }
      </span>
    );
    const gasUsed = transaction.gasUsed
      ? (
        <span className={ styles.highlight }>
          <FormattedMessage
            id='ui.methodDecoding.gasUsed'
            defaultMessage=' ({gas} gas used)'
            values={ {
              gas: transaction.gasUsed.toFormat(0)
            } }
          />
        </span>
      )
      : '';

    return (
      <div className={ styles.gasDetails }>
        <FormattedMessage
          id='ui.methodDecoding.txValues'
          defaultMessage='{historic, select, true {Provided} false {Provides}} {gasProvided}{gasUsed} for a total transaction value of {totalEthValue}'
          values={ {
            historic,
            gasProvided,
            gasUsed,
            totalEthValue
          } }
        />
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

    const blockCondition = new BigNumber(condition.block || 0);

    if (blockCondition.gt(0)) {
      const blockNumber = (
        <span className={ styles.highlight }>
          #{ blockCondition.toFormat(0) }
        </span>
      );

      return (
        <div>
          <FormattedMessage
            id='ui.methodDecoding.condition.block'
            defaultMessage='{historic, select, true {Will be submitted} false {To be submitted}} at block {blockNumber}'
            values={ {
              historic,
              blockNumber
            } }
          />
        </div>
      );
    }

    if (condition.time) {
      const timestamp = (
        <span className={ styles.highlight }>
          { moment(condition.time).format('LLLL') }
        </span>
      );

      return (
        <div>
          <FormattedMessage
            id='ui.methodDecoding.condition.time'
            defaultMessage='{historic, select, true {Will be submitted} false {To be submitted}} {timestamp}'
            values={ {
              historic,
              timestamp
            } }
          />
        </div>
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
    const value = api.util.hexToAscii(transaction.input || transaction.data);

    return {
      value,
      valid: ASCII_INPUT.test(value)
    };
  }

  renderInputValue () {
    const { compact, transaction } = this.props;

    if (compact) {
      return null;
    }

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

    const inputDesc = (
      <span
        onClick={ this.toggleInputType }
        className={ [ styles.clickable, styles.noSelect ].join(' ') }
      >
        {
          type === 'ascii'
            ? (
              <FormattedMessage
                id='ui.methodDecoding.input.input'
                defaultMessage='input'
              />
            )
            : (
              <FormattedMessage
                id='ui.methodDecoding.input.data'
                defaultMessage='data'
              />
            )
        }
      </span>
    );
    const inputValue = (
      <span
        onClick={ this.toggleInputExpand }
        className={ expandable ? styles.clickable : '' }
      >
        <code className={ styles.inputData }>
          { textToShow }
        </code>
      </span>
    );

    return (
      <div className={ styles.details }>
        <FormattedMessage
          id='ui.methodDecoding.input.withInput'
          defaultMessage='with the {inputDesc} {inputValue}'
          values={ {
            inputDesc,
            inputValue
          } }
        />
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
            <FormattedMessage
              id='ui.methodDecoding.token.transfer'
              defaultMessage='{historic, select, true {Transferred} false {Will transfer}} {value} to {address}'
              values={ {
                historic,
                value: (
                  <span className={ styles.highlight }>
                    { this.renderTokenValue(value.value) }
                  </span>
                ),
                address: this.renderAddressName(address)
              } }
            />
          </div>
        );
    }
  }

  renderDeploy () {
    const { compact, historic, transaction } = this.props;
    const { methodInputs } = this.state;
    const { value } = transaction;

    if (!historic) {
      return (
        <div className={ styles.details }>
          <FormattedMessage
            id='ui.methodDecoding.deploy.willDeploy'
            defaultMessage='Will deploy a contract'
          />
          {
            value && value.gt(0)
            ? (
              <FormattedMessage
                id='ui.methodDecoding.deploy.withValue'
                defaultMessage=', sending {value}'
                values={ {
                  value: this.renderEtherValue(value)
                } }
              />
            )
            : null
          }
        </div>
      );
    }

    return (
      <div className={ styles.details }>
        <div>
          <FormattedMessage
            id='ui.methodDecoding.deploy.address'
            defaultMessage='Deployed a contract at address '
          />
        </div>
        { this.renderAddressName(transaction.creates, false) }
        {
          !compact && methodInputs && methodInputs.length
          ? (
            <div>
              <FormattedMessage
                id='ui.methodDecoding.deploy.params'
                defaultMessage='with the following parameters:'
              />
              <div className={ styles.inputs }>
                { this.renderInputs() }
              </div>
            </div>
          )
          : null
        }
      </div>
    );
  }

  renderValueReceipt () {
    const { historic, transaction } = this.props;
    const { isContract } = this.state;

    const valueEth = (
      <span className={ styles.highlight }>
        { this.renderEtherValue(transaction.value) }
      </span>
    );
    const aContract = isContract
      ? (
        <FormattedMessage
          id='ui.methodDecoding.receive.contract'
          defaultMessage='the contract '
        />
      )
      : '';

    return (
      <div className={ styles.details }>
        <FormattedMessage
          id='ui.methodDecoding.receive.info'
          defaultMessage='{historic, select, true {Received} false {Will receive}} {valueEth} from {aContract}{address}'
          values={ {
            historic,
            valueEth,
            aContract,
            address: this.renderAddressName(transaction.from)
          } }
        />
        { this.renderInputValue() }
      </div>
    );
  }

  renderValueTransfer () {
    const { historic, transaction } = this.props;
    const { isContract } = this.state;

    const valueEth = (
      <span className={ styles.highlight }>
        { this.renderEtherValue(transaction.value) }
      </span>
    );
    const aContract = isContract
      ? (
        <FormattedMessage
          id='ui.methodDecoding.transfer.contract'
          defaultMessage='the contract '
        />
      )
      : '';

    return (
      <div className={ styles.details }>
        <FormattedMessage
          id='ui.methodDecoding.transfer.info'
          defaultMessage='{historic, select, true {Transferred} false {Will transfer}} {valueEth} to {aContract}{address}'
          values={ {
            historic,
            valueEth,
            aContract,
            address: this.renderAddressName(transaction.to)
          } }
        />
        { this.renderInputValue() }
      </div>
    );
  }

  renderSignatureMethod () {
    const { compact, historic, transaction } = this.props;
    const { methodName, methodInputs } = this.state;

    const showInputs = !compact && methodInputs && methodInputs.length > 0;
    const showEth = !!(transaction.value && transaction.value.gt(0));

    const method = (
      <span className={ styles.name }>
        { methodName }
      </span>
    );
    const ethValue = showEth && (
      <span className={ styles.highlight }>
        { this.renderEtherValue(transaction.value) }
      </span>
    );

    return (
      <div className={ styles.details }>
        <div className={ styles.description }>
          <FormattedMessage
            id='ui.methodDecoding.signature.info'
            defaultMessage='{historic, select, true {Executed} false {Will execute}} the {method} function on the contract {address} {showEth, select, true {transferring {ethValue}} false {}} {showInputs, select, false {} true {passing the following {inputLength, plural, one {parameter} other {parameters}}}}'
            values={ {
              historic,
              method,
              ethValue,
              showEth,
              showInputs,
              address: this.renderAddressName(transaction.to),
              inputLength: methodInputs.length
            } }
          />
        </div>
        {
          showInputs
          ? (
            <div className={ styles.inputs }>
              { this.renderInputs() }
            </div>
          )
          : null
        }
      </div>
    );
  }

  renderUnknownMethod () {
    const { historic, transaction } = this.props;

    const method = (
      <span className={ styles.name }>
        an unknown/unregistered
      </span>
    );
    const ethValue = (
      <span className={ styles.highlight }>
        { this.renderEtherValue(transaction.value) }
      </span>
    );

    return (
      <div className={ styles.details }>
        <FormattedMessage
          id='ui.methodDecoding.unknown.info'
          defaultMessage='{historic, select, true {Executed} false {Will execute}} the {method} on the contract {address} transferring {ethValue}.'
          values={ {
            historic,
            method,
            ethValue,
            address: this.renderAddressName(transaction.to)
          } }
        />
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
          value={ input.value }
        />
      );
    });

    return inputs;
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
  const { tokens } = initState;
  const { transaction } = initProps;

  const token = Object.values(tokens).find((token) => token.address === transaction.to);

  return () => {
    return { token };
  };
}

export default connect(
  mapStateToProps,
  null
)(MethodDecoding);
