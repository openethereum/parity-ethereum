// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import Store from '../../store';
import * as RequestsActions from '~/redux/providers/signerActions';
import { Container } from '~/ui';

import RequestPending from '../../components/RequestPending';

import styles from './embedded.css';

class Embedded extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    actions: PropTypes.shape({
      startConfirmRequest: PropTypes.func.isRequired,
      startRejectRequest: PropTypes.func.isRequired
    }).isRequired,
    gasLimit: PropTypes.object.isRequired,
    isTest: PropTypes.bool.isRequired,
    signer: PropTypes.shape({
      finished: PropTypes.array.isRequired,
      pending: PropTypes.array.isRequired
    }).isRequired
  };

  store = new Store(this.context.api);

  render () {
    return (
      <Container style={ { background: 'transparent' } }>
        <div className={ styles.signer }>
          { this.renderPendingRequests() }
        </div>
      </Container>
    );
  }

  renderPendingRequests () {
    const { signer } = this.props;
    const { pending } = signer;

    if (!pending.length) {
      return (
        <div className={ styles.none }>
          There are currently no pending requests awaiting your confirmation
        </div>
      );
    }

    const items = pending.sort(this._sortRequests).map(this.renderPending);

    return (
      <div>
        { items }
      </div>
    );
  }

  renderPending = (data, index) => {
    const { actions, gasLimit, isTest } = this.props;
    const { date, id, isSending, payload } = data;

    return (
      <RequestPending
        className={ styles.request }
        date={ date }
        focus={ index === 0 }
        gasLimit={ gasLimit }
        id={ id }
        isSending={ isSending }
        isTest={ isTest }
        key={ id }
        onConfirm={ actions.startConfirmRequest }
        onReject={ actions.startRejectRequest }
        payload={ payload }
        store={ this.store }
      />
    );
  }

  _sortRequests = (a, b) => {
    return new BigNumber(b.id).cmp(a.id);
  }
}

function mapStateToProps (state) {
  const { gasLimit, isTest } = state.nodeStatus;
  const { actions, signer } = state;

  return {
    actions,
    gasLimit,
    isTest,
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
)(Embedded);
