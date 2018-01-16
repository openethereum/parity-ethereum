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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import * as RequestsActions from '@parity/shared/lib/redux/providers/signerActions';
import Container from '@parity/ui/lib/Container';

import PendingList from '../PendingList';

import styles from './embedded.css';

const CONTAINER_STYLE = {
  background: 'transparent'
};

class Embedded extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    actions: PropTypes.shape({
      startConfirmRequest: PropTypes.func.isRequired,
      startRejectRequest: PropTypes.func.isRequired
    }).isRequired,
    gasLimit: PropTypes.object.isRequired,
    netVersion: PropTypes.string.isRequired,
    pending: PropTypes.array.isRequired
  };

  render () {
    const { accounts, actions, gasLimit, netVersion, pending } = this.props;

    return (
      <Container style={ CONTAINER_STYLE }>
        <PendingList
          accounts={ accounts }
          className={ styles.signer }
          gasLimit={ gasLimit }
          netVersion={ netVersion }
          onConfirm={ actions.startConfirmRequest }
          onReject={ actions.startRejectRequest }
          pendingItems={ pending }
        />
      </Container>
    );
  }
}

function mapStateToProps (state) {
  const { gasLimit, netVersion } = state.nodeStatus;
  const { accounts } = state.personal;
  const { pending } = state.signer;
  const { actions } = state;

  // TODO: Use the pending store & actions inside that store to confirm/reject, get rid of the Redux interface

  return {
    accounts,
    actions,
    gasLimit,
    netVersion,
    pending
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
