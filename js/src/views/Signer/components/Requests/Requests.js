import React, { Component, PropTypes } from 'react';
import { RequestPendingWeb3, RequestFinishedWeb3 } from 'dapps-react-components';
import styles from './Requests.css';

export default class Requests extends Component {

  static propTypes = {
    requests: PropTypes.shape({
      pending: PropTypes.array.isRequired,
      finished: PropTypes.array.isRequired
    }).isRequired,
    actions: PropTypes.shape({
      startConfirmRequest: PropTypes.func.isRequired,
      startRejectRequest: PropTypes.func.isRequired
    }).isRequired
  };

  render () {
    const { pending, finished } = this.props.requests;

    if (!pending.length && !finished.length) {
      return this.renderNoRequestsMsg();
    }

    return (
      <div>
        { this.renderFinishedRequests() }
        { this.renderPendingRequests() }
      </div>
    );
  }

  renderPendingRequests () {
    const { requests } = this.props;
    if (!requests.pending.length) {
      return;
    }

    return (
      <div>
        <h2>Pending Requests</h2>
        <div>{ requests.pending.map(data => this.renderPending(data)) }</div>
      </div>
    );
  }

  renderFinishedRequests () {
    const { finished } = this.props.requests;
    if (!finished.length) {
      return;
    }

    return (
      <div>
        <h2>Finished Requests</h2>
        <div>{ finished.map(data => this.renderFinished(data)) }</div>
      </div>
    );
  }

  renderPending (data) {
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

  renderFinished (data) {
    const { payload, id, result, msg, status, error } = data;

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
        />
    );
  }

  renderNoRequestsMsg () {
    return (
      <div className={ styles.noRequestsMsg }>
        <h3>There are no requests requiring your confirmation.</h3>
      </div>
    );
  }

}
