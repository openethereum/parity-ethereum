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

import HardwareStore from '~/mobx/hardwareStore';
import { Button, GasPriceEditor } from '~/ui';

import TransactionMainDetails from '../TransactionMainDetails';
import TransactionPendingForm from '../TransactionPendingForm';

import styles from './transactionPending.css';

import * as tUtil from '../util/transaction';

@observer
class TransactionPending extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    className: PropTypes.string,
    date: PropTypes.instanceOf(Date).isRequired,
    focus: PropTypes.bool,
    gasLimit: PropTypes.object,
    id: PropTypes.object.isRequired,
    isSending: PropTypes.bool.isRequired,
    netVersion: PropTypes.string.isRequired,
    nonce: PropTypes.number,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    origin: PropTypes.any,
    signerstore: PropTypes.object.isRequired,
    transaction: PropTypes.shape({
      condition: PropTypes.object,
      data: PropTypes.string,
      from: PropTypes.string.isRequired,
      gas: PropTypes.object.isRequired,
      gasPrice: PropTypes.object.isRequired,
      to: PropTypes.string,
      value: PropTypes.object.isRequired
    }).isRequired
  };

  static defaultProps = {
    focus: false,
    origin: {
      type: 'unknown',
      details: ''
    }
  };

  gasStore = new GasPriceEditor.Store(this.context.api, {
    condition: this.props.transaction.condition,
    gas: this.props.transaction.gas.toFixed(),
    gasLimit: this.props.gasLimit,
    gasPrice: this.props.transaction.gasPrice.toFixed()
  });

  hwstore = HardwareStore.get(this.context.api);

  componentWillMount () {
    const { signerstore, transaction } = this.props;
    const { from, gas, gasPrice, to, value } = transaction;

    const fee = tUtil.getFee(gas, gasPrice); // BigNumber object
    const gasPriceEthmDisplay = tUtil.getEthmFromWeiDisplay(gasPrice);
    const gasToDisplay = tUtil.getGasDisplay(gas);
    const totalValue = tUtil.getTotalValue(fee, value);

    this.setState({ gasPriceEthmDisplay, totalValue, gasToDisplay });
    this.gasStore.setEthValue(value);
    signerstore.fetchBalances([from, to]);
  }

  render () {
    return this.gasStore.isEditing
      ? this.renderTxEditor()
      : this.renderTransaction();
  }

  renderTransaction () {
    const { accounts, className, focus, id, isSending, netVersion, origin, signerstore, transaction } = this.props;
    const { totalValue } = this.state;
    const { balances, externalLink } = signerstore;
    const { from, value } = transaction;
    const fromBalance = balances[from];
    const account = accounts[from] || {};
    const disabled = account.hardware && !this.hwstore.isConnected(from);

    return (
      <div className={ `${styles.container} ${className}` }>
        <TransactionMainDetails
          className={ styles.transactionDetails }
          disabled={ disabled }
          externalLink={ externalLink }
          from={ from }
          fromBalance={ fromBalance }
          gasStore={ this.gasStore }
          id={ id }
          netVersion={ netVersion }
          origin={ origin }
          totalValue={ totalValue }
          transaction={ transaction }
          value={ value }
        />
        <TransactionPendingForm
          account={ account }
          address={ from }
          disabled={ disabled }
          focus={ focus }
          isSending={ isSending }
          onConfirm={ this.onConfirm }
          onReject={ this.onReject }
        />
      </div>
    );
  }

  renderTxEditor () {
    const { className } = this.props;

    return (
      <div className={ `${styles.container} ${className}` }>
        <GasPriceEditor store={ this.gasStore }>
          <Button
            label={
              <FormattedMessage
                id='signer.txPending.buttons.viewToggle'
                defaultMessage='view transaction'
              />
            }
            onClick={ this.toggleGasEditor }
          />
        </GasPriceEditor>
      </div>
    );
  }

  onConfirm = (data) => {
    const { id, transaction } = this.props;
    const { password, wallet } = data;
    const { condition, gas, gasPrice } = this.gasStore.overrideTransaction(transaction);

    const options = {
      gas,
      gasPrice,
      id,
      password,
      wallet
    };

    if (condition && (condition.block || condition.time)) {
      options.condition = condition;
    }

    this.props.onConfirm(options);
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }

  toggleGasEditor = () => {
    this.gasStore.setEditing(false);
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return {
    accounts
  };
}

export default connect(
  mapStateToProps,
  null
)(TransactionPending);
