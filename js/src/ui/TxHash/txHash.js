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
import ShortenedHash from '../ShortenedHash';

import styles from './txHash.css';

class TxHash extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    hash: PropTypes.string.isRequired,
    isTest: PropTypes.bool,
    maxConfirmations: PropTypes.number,
    summary: PropTypes.bool
  }

  static defaultProps = {
    maxConfirmations: 10
  };

  state = {
    blockNumber: new BigNumber(0),
    subscriptionId: null,
    transaction: null
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
    const { hash, isTest, summary } = this.props;

    const hashLink = (
      <a href={ txLink(hash, isTest) } target='_blank'>
        <ShortenedHash data={ hash } />
      </a>
    );

    return (
      <div>
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

  renderConfirmations () {
    const { maxConfirmations } = this.props;
    const { blockNumber, transaction } = this.state;

    if (!(transaction && transaction.blockNumber && transaction.blockNumber.gt(0))) {
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

    const confirmations = blockNumber.minus(transaction.blockNumber).plus(1);
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

    if (error || !hash || /^(0x)?0*$/.test(hash)) {
      return;
    }

    return api.eth
      .getTransactionReceipt(hash)
      .then((transaction) => {
        this.setState({
          blockNumber,
          transaction
        });
      })
      .catch((error) => {
        console.warn('onBlockNumber', error);
        this.setState({ blockNumber });
      });
  }
}

function mapStateToProps (state) {
  const { isTest } = state.nodeStatus;

  return { isTest };
}

export default connect(
  mapStateToProps,
  null
)(TxHash);
