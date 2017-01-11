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

import React, { Component, PropTypes } from 'react';

import BackIcon from 'material-ui/svg-icons/navigation/arrow-back';

import TransactionPendingFormConfirm from './TransactionPendingFormConfirm';
import TransactionPendingFormReject from './TransactionPendingFormReject';
import styles from './transactionPendingForm.css';

export default class TransactionPendingForm extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    isSending: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    className: PropTypes.string,
    focus: PropTypes.bool
  };

  static defaultProps = {
    focus: false
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
    const { address, focus, isSending, onConfirm, onReject } = this.props;

    if (this.state.isRejectOpen) {
      return (
        <TransactionPendingFormReject
          onReject={ onReject } />
      );
    }

    return (
      <TransactionPendingFormConfirm
        address={ address }
        focus={ focus }
        isSending={ isSending }
        onConfirm={ onConfirm }
      />
    );
  }

  renderRejectToggle () {
    const { isRejectOpen } = this.state;
    let html;

    if (!isRejectOpen) {
      html = <span>reject transaction</span>;
    } else {
      html = <span><BackIcon />{ "I've changed my mind" }</span>;
    }

    return (
      <a
        className={ styles.rejectToggle }
        onClick={ this.onToggleReject }>
        { html }
      </a>
    );
  }

  onToggleReject = () => {
    const { isRejectOpen } = this.state;

    this.setState({
      isRejectOpen: !isRejectOpen
    });
  }
}
