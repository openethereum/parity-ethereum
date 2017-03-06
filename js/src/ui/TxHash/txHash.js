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
import { LinearProgress } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { txLink } from '~/3rdparty/etherscan/links';
import Warning from '~/ui/Warning';
import { DEFAULT_GAS } from '~/util/constants';

import ShortenedHash from '../ShortenedHash';
import styles from './txHash.css';

class TxHash extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    hash: PropTypes.string.isRequired,
    maxConfirmations: PropTypes.number,
    netVersion: PropTypes.string.isRequired,
    summary: PropTypes.bool
  }

  static defaultProps = {
    maxConfirmations: 10
  };

  state = {
    blockNumber: new BigNumber(0),
    isRecipientContract: false,
    subscriptionId: null,
    transaction: null,
    transactionReceipt: null
  }

  componentWillMount () {
    this.fetchTransaction();
  }

  componentWillReceiveProps (nextProps) {
    const prevHash = this.props.hash;
    const nextHash = nextProps.hash;

    if (prevHash !== nextHash) {
      this.fetchTransaction(nextProps);
    }
  }

  /**
   * Get the sent transaction data
   */
  fetchTransaction (props = this.props) {
    const { hash } = props;

    if (!hash) {
      return;
    }

    this.context.api.eth
      .getTransactionByHash(hash)
      .then((transaction) => {
        this.setState({ transaction });

        return this.fetchRecipientCode(transaction);
      });
  }

  fetchRecipientCode (transaction) {
    if (!transaction || !transaction.to) {
      return;
    }

    this.context.api.eth
      .getCode(transaction.to)
      .then((code) => {
        const isRecipientContract = code && !/^(0x)?0*$/.test(code);

        this.setState({ isRecipientContract });
      })
      .catch((error) => {
        console.error('fetchRecipientCode', error);
      });
  }

  componentDidMount () {
    const { api } = this.context;

    return api.subscribe('eth_blockNumber', this.onBlockNumber).then((subscriptionId) => {
      this.setState({ subscriptionId });
    });
  }

  componentWillUnmount () {
    const { api } = this.context;
    const { subscriptionId } = this.state;

    return api.unsubscribe(subscriptionId);
  }

  render () {
    const { hash, netVersion, summary } = this.props;

    const hashLink = (
      <a href={ txLink(hash, false, netVersion) } target='_blank'>
        <ShortenedHash data={ hash } />
      </a>
    );

    return (
      <div>
        { this.renderWarning() }
        <p>{
          summary
            ? hashLink
            : <FormattedMessage
              id='ui.txHash.posted'
              defaultMessage='The transaction has been posted to the network with a hash of {hashLink}'
              values={ { hashLink } }
              />
        }</p>
        { this.renderConfirmations() }
      </div>
    );
  }

  renderWarning () {
    const { isRecipientContract, transaction, transactionReceipt } = this.state;

    if (!(transactionReceipt && transactionReceipt.blockNumber && transactionReceipt.blockNumber.gt(0))) {
      return null;
    }

    const { gas, input } = transaction;
    const { gasUsed = new BigNumber(0) } = transactionReceipt;

    const isOog = gasUsed.gte(gas);

    // Skip OOG check if a simple transaction to a non-contract account
    // @see: https://github.com/ethcore/parity/issues/4550
    const skipOogCheck = gasUsed.eq(DEFAULT_GAS) && (!input || input === '0x') && !isRecipientContract;

    if (!isOog || skipOogCheck) {
      return null;
    }

    return (
      <Warning
        warning={
          <FormattedMessage
            id='ui.txHash.oog'
            defaultMessage='The transaction might have gone out of gas. Try again with more gas.'
          />
        }
      />
    );
  }

  renderConfirmations () {
    const { maxConfirmations } = this.props;
    const { blockNumber, transactionReceipt } = this.state;

    if (!(transactionReceipt && transactionReceipt.blockNumber && transactionReceipt.blockNumber.gt(0))) {
      return (
        <div className={ styles.confirm }>
          <LinearProgress
            className={ styles.progressbar }
            color='white'
            mode='indeterminate'
          />
          <div className={ styles.progressinfo }>
            <FormattedMessage
              id='ui.txHash.waiting'
              defaultMessage='waiting for confirmations'
            />
          </div>
        </div>
      );
    }

    const confirmations = blockNumber.minus(transactionReceipt.blockNumber).plus(1);
    const value = Math.min(confirmations.toNumber(), maxConfirmations);

    let count = confirmations.toFormat(0);

    if (confirmations.lte(maxConfirmations)) {
      count = `${count}/${maxConfirmations}`;
    }

    return (
      <div className={ styles.confirm }>
        <LinearProgress
          className={ styles.progressbar }
          min={ 0 }
          max={ maxConfirmations }
          value={ value }
          color='white'
          mode='determinate'
        />
        <div className={ styles.progressinfo }>
          <abbr title={ `block #${blockNumber.toFormat(0)}` }>
            <FormattedMessage
              id='ui.txHash.confirmations'
              defaultMessage='{count} {value, plural, one {confirmation} other {confirmations}}'
              values={ {
                count,
                value
              } }
            />
          </abbr>
        </div>
      </div>
    );
  }

  onBlockNumber = (error, blockNumber) => {
    const { api } = this.context;
    const { hash } = this.props;

    const nextState = { blockNumber };

    if (error || !hash || /^(0x)?0*$/.test(hash)) {
      return this.setState(nextState);
    }

    return api.eth
      .getTransactionReceipt(hash)
      .then((transactionReceipt) => {
        nextState.transactionReceipt = transactionReceipt;
      })
      .catch((error) => {
        console.error('onBlockNumber', error);
      })
      .then(() => {
        this.setState(nextState);
      });
  }
}

function mapStateToProps (state) {
  const { netVersion } = state.nodeStatus;

  return {
    netVersion
  };
}

export default connect(
  mapStateToProps,
  null
)(TxHash);
