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
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Store from '../../store';
import { newError } from '~/redux/actions';
import { startConfirmRequest, startRejectRequest } from '~/redux/providers/signerActions';
import { Container, Page, TxList } from '~/ui';

import RequestPending from '../../components/RequestPending';

import styles from './requestsPage.css';

@observer
class RequestsPage extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    gasLimit: PropTypes.object.isRequired,
    netVersion: PropTypes.string.isRequired,
    startConfirmRequest: PropTypes.func.isRequired,
    startRejectRequest: PropTypes.func.isRequired,

    blockNumber: PropTypes.object,
    newError: PropTypes.func,
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
    return new BigNumber(a.id).cmp(b.id);
  }

  renderLocalQueue () {
    const { localHashes } = this.store;
    const { blockNumber, newError } = this.props;

    if (!localHashes.length) {
      return null;
    }

    return (
      <Container
        title={
          <FormattedMessage
            id='signer.requestsPage.queueTitle'
            defaultMessage='Local Transactions'
          />
        }
      >
        <TxList
          address=''
          blockNumber={ blockNumber }
          hashes={ localHashes }
          onNewError={ newError }
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
            <FormattedMessage
              id='signer.requestsPage.noPending'
              defaultMessage='There are no requests requiring your confirmation.'
            />
          </div>
        </Container>
      );
    }

    const items = pending.sort(this._sortRequests).map(this.renderPending);

    return (
      <Container
        title={
          <FormattedMessage
            id='signer.requestsPage.pendingTitle'
            defaultMessage='Pending Signature Authorization'
          />
        }
      >
        { items }
      </Container>
    );
  }

  renderPending = (data, index) => {
    const { startConfirmRequest, startRejectRequest, gasLimit, netVersion } = this.props;
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
        onConfirm={ startConfirmRequest }
        onReject={ startRejectRequest }
        origin={ origin }
        payload={ payload }
        signerStore={ this.store }
      />
    );
  }
}

function mapStateToProps (state) {
  const { gasLimit, netVersion, blockNumber } = state.nodeStatus;
  const { signer } = state;

  return {
    blockNumber,
    gasLimit,
    netVersion,
    signer
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError,
    startConfirmRequest,
    startRejectRequest
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(RequestsPage);
