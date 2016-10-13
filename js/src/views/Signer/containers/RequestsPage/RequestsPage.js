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
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import * as RequestsActions from '../../../../redux/providers/signerActions';
import { Container, ContainerTitle } from '../../../../ui';

import { RequestPendingWeb3, RequestFinishedWeb3 } from '../../components';

import styles from './RequestsPage.css';

class RequestsPage extends Component {
  static propTypes = {
    signer: PropTypes.shape({
      pending: PropTypes.array.isRequired,
      finished: PropTypes.array.isRequired
    }).isRequired,
    actions: PropTypes.shape({
      startConfirmRequest: PropTypes.func.isRequired,
      startRejectRequest: PropTypes.func.isRequired
    }).isRequired
  };

  render () {
    const { pending, finished } = this.props.signer;

    if (!pending.length && !finished.length) {
      return this.renderNoRequestsMsg();
    }

    return (
      <div>
        { this.renderPendingRequests() }
        { this.renderFinishedRequests() }
      </div>
    );
  }

  _sortRequests = (a, b) => {
    return new BigNumber(b.id).cmp(a.id);
  }

  renderPendingRequests () {
    const { pending } = this.props.signer;

    if (!pending.length) {
      return;
    }

    const items = pending.sort(this._sortRequests).map(this.renderPending);

    return (
      <Container>
        <ContainerTitle title='Pending Requests' />
        <div className={ styles.items }>
          { items }
        </div>
      </Container>
    );
  }

  renderFinishedRequests () {
    const { finished } = this.props.signer;

    if (!finished.length) {
      return;
    }

    const items = finished.sort(this._sortRequests).map(this.renderFinished);

    return (
      <Container>
        <ContainerTitle title='Finished Requests' />
        <div className={ styles.items }>
          { items }
        </div>
      </Container>
    );
  }

  renderPending = (data) => {
    const { actions } = this.props;
    const { payload, id, isSending, date } = data;

    return (
      <RequestPendingWeb3
        className={ styles.request }
        onConfirm={ actions.startConfirmRequest }
        onReject={ actions.startRejectRequest }
        isSending={ isSending || false }
        key={ id }
        id={ id }
        payload={ payload }
        date={ date }
      />
    );
  }

  renderFinished = (data) => {
    const { payload, id, result, msg, status, error, date } = data;

    return (
      <RequestFinishedWeb3
        className={ styles.request }
        result={ result }
        key={ id }
        id={ id }
        msg={ msg }
        status={ status }
        error={ error }
        payload={ payload }
        date={ date }
        />
    );
  }

  renderNoRequestsMsg () {
    return (
      <Container>
        <div className={ styles.noRequestsMsg }>
          There are no requests requiring your confirmation.
        </div>
      </Container>
    );
  }
}

function mapStateToProps (state) {
  return state;
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
