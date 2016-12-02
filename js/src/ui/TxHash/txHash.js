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
import { LinearProgress } from 'material-ui';

import { txLink } from '../../3rdparty/etherscan/links';
import ShortenedHash from '../ShortenedHash';

import styles from './txHash.css';

class TxHash extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    hash: PropTypes.string.isRequired,
    isTest: PropTypes.bool,
    summary: PropTypes.bool,
    maxConfirmations: PropTypes.number
  }

  static defaultProps = {
    maxConfirmations: 10
  };

  state = {
    blockNumber: new BigNumber(0),
    transaction: null,
    subscriptionId: null
  }

  componentDidMount () {
    const { api } = this.context;

    api.subscribe('eth_blockNumber', this.onBlockNumber).then((subscriptionId) => {
      this.setState({ subscriptionId });
    });
  }

  componentWillUnmount () {
    const { api } = this.context;
    const { subscriptionId } = this.state;

    api.unsubscribe(subscriptionId);
  }

  render () {
    const { hash, isTest, summary } = this.props;

    const link = (
      <a href={ txLink(hash, isTest) } target='_blank'>
        <ShortenedHash data={ hash } />
      </a>
    );

    let header = (
      <p>The transaction has been posted to the network, with a hash of { link }.</p>
    );
    if (summary) {
      header = (<p>{ link }</p>);
    }

    return (
      <div>
        { header }
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
          <div className={ styles.progressinfo }>waiting for confirmations</div>
        </div>
      );
    }

    const confirmations = blockNumber.minus(transaction.blockNumber).plus(1);
    const value = Math.min(confirmations.toNumber(), maxConfirmations);
    let count;
    if (confirmations.gt(maxConfirmations)) {
      count = confirmations.toFormat(0);
    } else {
      count = confirmations.toFormat(0) + `/${maxConfirmations}`;
    }
    const unit = value === 1 ? 'confirmation' : 'confirmations';

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
          <abbr title={ `block #${blockNumber.toFormat(0)}` }>{ count } { unit }</abbr>
        </div>
      </div>
    );
  }

  onBlockNumber = (error, blockNumber) => {
    const { api } = this.context;
    const { hash } = this.props;

    if (error) {
      return;
    }

    this.setState({ blockNumber });

    api.eth
      .getTransactionReceipt(hash)
      .then((transaction) => {
        this.setState({ transaction });
      })
      .catch((error) => {
        console.warn('onBlockNumber', error);
      });
  }
}

function mapStateToProps (state) {
  const { isTest } = state.nodeStatus;

  return { isTest };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TxHash);
