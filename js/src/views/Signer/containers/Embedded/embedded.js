import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { RequestPendingWeb3 } from '../../components';
import * as RequestsActions from '../../actions/requests';

import styles from './embedded.js';

class Embedded extends Component {
  static propTypes = {
    signerRequests: PropTypes.shape({
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
      <div className={ styles.signer }>
        { this.renderPendingRequests() }
      </div>
    );
  }

  renderPendingRequests () {
    const { signerRequests } = this.props;
    const { pending } = signerRequests;

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
    const { payload, id, isSending } = data;

    return (
      <RequestPendingWeb3
        className={ styles.request }
        onConfirm={ actions.startConfirmRequest }
        onReject={ actions.startRejectRequest }
        isSending={ isSending || false }
        key={ id }
        id={ id }
        payload={ payload }
      />
    );
  }
}

function mapStateToProps (state) {
  const { actions, signerRequests } = state;

  return {
    actions,
    signerRequests
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
