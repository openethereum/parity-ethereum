import React, { Component } from 'react';

import TransactionPendingForm from '../../../TransactionPendingForm';

export default class TransactionPendingFormPage extends Component {

  state = {
    chosenAction: null,
    password: null
  }

  render () {
    return (
      <div>
        <h1>Transaction Pending Form</h1>
        { this.renderForm() }
        { this.renderChosenAction() }
      </div>
    );
  }

  renderForm () {
    return (
      <TransactionPendingForm
        onConfirm={ this.onConfirm }
        onReject={ this.onReject }
      />
    );
  }

  renderChosenAction () {
    const { chosenAction } = this.state;
    if (!chosenAction) {
      return;
    }

    return (
      <p>
        You have
        <strong> { chosenAction } </strong>
        this pending transaction
        { this.renderWithPassword() }
        .
      </p>
    );
  }

  // rejecting transaction has no password
  renderWithPassword () {
    const { password } = this.state;
    if (!password) {
      return;
    }

    return ' with password ' + password;
  }

  onConfirm = password => {
    this.setState({
      password,
      chosenAction: 'confirmed'
    });
  }

  onReject = () => {
    this.setState({
      password: null,
      chosenAction: 'rejected'
    });
  }

}
