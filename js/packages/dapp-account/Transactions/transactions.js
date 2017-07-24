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
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { Container, TxList, Loading } from '@parity/ui';

import Store from './store';
import styles from './transactions.css';

@observer
class Transactions extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    traceMode: PropTypes.bool
  }

  store = new Store(this.context.api);

  componentWillMount () {
    this.store.updateProps(this.props);
  }

  componentWillReceiveProps (newProps) {
    if (this.props.traceMode === undefined && newProps.traceMode !== undefined) {
      this.store.updateProps(newProps);
      return;
    }

    const hasChanged = ['address', 'netVersion']
      .map(key => newProps[key] !== this.props[key])
      .reduce((truth, keyTruth) => truth || keyTruth, false);

    if (hasChanged) {
      this.store.updateProps(newProps);
    }
  }

  render () {
    return (
      <Container
        title={
          <FormattedMessage
            id='account.transactions.title'
            defaultMessage='transactions'
          />
        }
      >
        { this.renderTransactionList() }
        { this.renderEtherscanFooter() }
      </Container>
    );
  }

  renderTransactionList () {
    const { address, isLoading, txHashes } = this.store;

    if (isLoading) {
      return (
        <Loading />
      );
    }

    return (
      <TxList
        address={ address }
        hashes={ txHashes }
      />
    );
  }

  renderEtherscanFooter () {
    const { isTracing } = this.store;

    if (isTracing) {
      return null;
    }

    return (
      <div className={ styles.etherscan }>
        <FormattedMessage
          id='account.transactions.poweredBy'
          defaultMessage='Transaction list powered by {etherscan}'
          values={ {
            etherscan: <a href='https://etherscan.io/' target='_blank'>etherscan.io</a>
          } }
        />
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { netVersion, traceMode } = state.nodeStatus;

  return {
    netVersion,
    traceMode
  };
}

export default connect(
  mapStateToProps,
  null
)(Transactions);
