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
import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';
import { observer } from 'mobx-react';

import Store from '../../store';
import * as RequestsActions from '~/redux/providers/signerActions';
import { Container, Page, TxList } from '~/ui';

import RequestPending from '../../components/RequestPending';

import styles from './requestsPage.css';

@observer
class RequestsPage extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    actions: PropTypes.shape({
      startConfirmRequest: PropTypes.func.isRequired,
      startRejectRequest: PropTypes.func.isRequired
    }).isRequired,
    gasLimit: PropTypes.object.isRequired,
    netVersion: PropTypes.string.isRequired,
    signer: PropTypes.shape({
      pending: PropTypes.array.isRequired,
      finished: PropTypes.array.isRequired
    }).isRequired
  };

  store = new Store(this.context.api, true);

  componentWillUnmount () {
    this.store.unsubscribe();
  }

  render () {
    return (
      <Page>
        <div>{ this.renderPendingRequests() }</div>
        <div>{ this.renderLocalQueue() }</div>
      </Page>
    );
  }

  _sortRequests = (a, b) => {
    return new BigNumber(b.id).cmp(a.id);
  }

  renderLocalQueue () {
    const { localHashes } = this.store;

    if (!localHashes.length) {
      return null;
    }

    return (
      <Container title='Local Transactions'>
        <TxList
          address=''
          hashes={ localHashes }
        />
      </Container>
    );
  }

  renderPendingRequests () {
    const { pending } = this.props.signer;

    if (!pending.length) {
      return (
        <Container>
          <div className={ styles.noRequestsMsg }>
            There are no requests requiring your confirmation.
          </div>
        </Container>
      );
    }

    const items = pending.sort(this._sortRequests).map(this.renderPending);

    return (
      <Container title='Pending Requests'>
        { items }
      </Container>
    );
  }

  renderPending = (data, index) => {
    const { actions, gasLimit, netVersion } = this.props;
    const { date, id, isSending, payload, origin } = data;

    return (
      <RequestPending
        className={ styles.request }
        date={ date }
        focus={ index === 0 }
        gasLimit={ gasLimit }
        id={ id }
        isSending={ isSending }
        netVersion={ netVersion }
        key={ id }
        onConfirm={ actions.startConfirmRequest }
        onReject={ actions.startRejectRequest }
        origin={ origin }
        payload={ payload }
        signerstore={ this.store }
      />
    );
  }
}

function mapStateToProps (state) {
  const { gasLimit, netVersion } = state.nodeStatus;
  const { actions, signer } = state;

  return {
    actions,
    gasLimit,
    netVersion,
    signer
  };
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators(RequestsActions, dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(RequestsPage);
