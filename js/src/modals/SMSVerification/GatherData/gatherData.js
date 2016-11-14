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

import React, { Component, PropTypes } from 'react';
import { Checkbox } from 'material-ui';
import InfoIcon from 'material-ui/svg-icons/action/info-outline';
import SuccessIcon from 'material-ui/svg-icons/navigation/check';
import ErrorIcon from 'material-ui/svg-icons/navigation/close';

import phone from 'phoneformat.js';

import { fromWei } from '../../../api/util/wei';
import { Form, Input } from '../../../ui';
import checkIfVerified from '../check-if-verified';
import checkIfRequested from '../check-if-requested';

import terms from '../terms-of-service';
import styles from './gatherData.css';

export default class GatherData extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.string.isRequired,
    contract: PropTypes.object.isRequired,
    data: PropTypes.object.isRequired,
    onData: PropTypes.func.isRequired,
    onDataIsValid: PropTypes.func.isRequired,
    onDataIsInvalid: PropTypes.func.isRequired
  }

  state = {
    init: true,
    numberIsValid: null,
    consentGiven: false
  };

  componentWillMount () {
    const { init } = this.state;
    if (init) {
      this.setState({ init: false });
      this.queryFee();
      this.checkIfCertified();
      this.checkIfRequested();
    }
  }

  render () {
    const { numberIsValid } = this.state;
    const { isVerified } = this.props.data;

    // TODO: proper legal text
    return (
      <Form>
        <p>The following steps will let you prove that you control both an account and a phone number.</p>
        <ol className={ styles.list }>
          <li>You send a verification request to a specific contract.</li>
          <li>Our server puts a puzzle into this contract.</li>
          <li>The code you receive via SMS is the solution to this puzzle.</li>
        </ol>
        { this.renderFee() }
        { this.renderCertified() }
        { this.renderRequested() }
        <Input
          label={ 'phone number' }
          hint={ 'the SMS will be sent to this number' }
          error={ numberIsValid ? null : 'invalid number' }
          disabled={ isVerified }
          onChange={ this.numberOnChange }
          onSubmit={ this.numberOnSubmit }
        />
        <Checkbox
          className={ styles.spacing }
          label={ 'I agree to the terms and conditions below.' }
          disabled={ isVerified }
          onCheck={ this.consentOnChange }
        />
        <div className={ styles.terms }>{ terms }</div>
      </Form>
    );
  }

  renderFee () {
    const { fee } = this.props.data;

    if (!fee) {
      return (<p>Fetching the fee…</p>);
    }
    return (
      <div className={ styles.container }>
        <InfoIcon />
        <p className={ styles.message }>The fee is { fromWei(fee).toFixed(3) } ETH.</p>
      </div>
    );
  }

  renderCertified () {
    const { isVerified } = this.props.data;

    if (isVerified) {
      return (
        <div className={ styles.container }>
          <ErrorIcon />
          <p className={ styles.message }>Your account is already verified.</p>
        </div>
      );
    }
    if (isVerified === false) {
      return (
        <div className={ styles.container }>
          <SuccessIcon />
          <p className={ styles.message }>Your account is not verified yet.</p>
        </div>
      );
    }
    return (<p className={ styles.message }>Checking if your account is verified…</p>);
  }

  renderRequested () {
    const { isVerified, hasRequested } = this.props.data;

    // If the account is verified, don't show that it has requested verification.
    if (isVerified) {
      return null;
    }

    if (hasRequested) {
      return (
        <div className={ styles.container }>
          <InfoIcon />
          <p className={ styles.message }>You already requested verification.</p>
        </div>
      );
    }
    if (hasRequested === false) {
      return (
        <div className={ styles.container }>
          <SuccessIcon />
          <p className={ styles.message }>You did not request verification yet.</p>
        </div>
      );
    }
    return (<p className={ styles.message }>Checking if you requested verification…</p>);
  }

  queryFee = () => {
    const { contract, onData } = this.props;

    contract.instance.fee.call()
    .then((fee) => {
      onData({ fee });
      this.onChange();
    })
    .catch((err) => {
      console.error('error fetching fee', err);
      this.onChange();
    });
  }

  checkIfCertified = () => {
    const { account, contract, onData } = this.props;

    checkIfVerified(contract, account)
    .then((isVerified) => {
      onData({ isVerified });
      this.onChange();
    })
    .catch((err) => {
      console.error('error checking if certified', err);
    });
  }

  checkIfRequested = () => {
    const { account, contract, onData } = this.props;

    checkIfRequested(contract, account)
    .then((hasRequested) => {
      onData({ hasRequested });
      this.onChange();
    })
    .catch((err) => {
      console.error('error checking if requested', err);
    });
  }

  numberOnSubmit = (value) => {
    this.numberOnChange(null, value);
    this.props.onData({ number: value });
  }

  numberOnChange = (_, value) => {
    this.setState({
      numberIsValid: phone.isValidNumber(value)
    }, this.onChange);
  }

  consentOnChange = (_, consentGiven) => {
    this.setState({
      consentGiven: !!consentGiven
    }, this.onChange);
    this.props.onData({ consent: consentGiven });
  }

  onChange = () => {
    const { fee, isVerified } = this.props.data;
    const { numberIsValid, consentGiven } = this.state;

    if (fee && numberIsValid && consentGiven && isVerified === false) {
      this.props.onDataIsValid();
    } else {
      this.props.onDataIsInvalid();
    }
  }
}
