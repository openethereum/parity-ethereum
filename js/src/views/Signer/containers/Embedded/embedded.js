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

import * as RequestsActions from '../../../../redux/providers/signerActions';
import { Container } from '../../../../ui';

import { RequestPendingWeb3 } from '../../components';

import styles from './embedded.css';

class Embedded extends Component {
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
      <div className={ styles.pending }>
        { items }
      </div>
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

  _sortRequests = (a, b) => {
    return new BigNumber(b.id).cmp(a.id);
  }
}

function mapStateToProps (state) {
  const { actions, signer } = state;

  return {
    actions,
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
