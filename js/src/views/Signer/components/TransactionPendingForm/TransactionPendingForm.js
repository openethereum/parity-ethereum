import React, { Component, PropTypes } from 'react';

import BackIcon from 'material-ui/svg-icons/navigation/arrow-back';

import TransactionPendingFormConfirm from '../TransactionPendingFormConfirm';
import TransactionPendingFormReject from '../TransactionPendingFormReject';
import styles from './TransactionPendingForm.css';

export default class TransactionPendingForm extends Component {

  static propTypes = {
    isSending: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    className: PropTypes.string
  };

  state = {
    isRejectOpen: false
  };

  render () {
    const { className } = this.props;

    return (
      <div className={ `${styles.container} ${className}` }>
        { this.renderForm() }
        { this.renderRejectToggle() }
      </div>
    );
  }

  renderForm () {
    const { isSending, onConfirm, onReject } = this.props;
    if (this.state.isRejectOpen) {
      return (
        <TransactionPendingFormReject onReject={ onReject } />
      );
    }

    return (
      <TransactionPendingFormConfirm onConfirm={ onConfirm } isSending={ isSending } />
    );
  }

  renderRejectToggle () {
    const { isRejectOpen } = this.state;
    let html;

    if (!isRejectOpen) {
      html = <span>reject</span>;
    } else {
      html = <span><BackIcon />I've changed my mind</span>;
    }

    return (
      <a
        onClick={ this.onToggleReject }
        className={ styles.rejectToggle }
      >
      { html }
      </a>
    );
  }

  onToggleReject = () => {
    const { isRejectOpen } = this.state;
    this.setState({ isRejectOpen: !isRejectOpen });
  }

}
