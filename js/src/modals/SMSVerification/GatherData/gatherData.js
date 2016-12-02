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
import nullable from '../../../util/nullable-proptype';
import BigNumber from 'bignumber.js';
import { Checkbox } from 'material-ui';
import InfoIcon from 'material-ui/svg-icons/action/info-outline';
import SuccessIcon from 'material-ui/svg-icons/navigation/check';
import ErrorIcon from 'material-ui/svg-icons/navigation/close';

import { fromWei } from '../../../api/util/wei';
import { Form, Input } from '../../../ui';

import { termsOfService } from '../../../3rdparty/sms-verification';
import styles from './gatherData.css';

export default class GatherData extends Component {
  static propTypes = {
    fee: React.PropTypes.instanceOf(BigNumber),
    isNumberValid: PropTypes.bool.isRequired,
    isVerified: nullable(PropTypes.bool.isRequired),
    hasRequested: nullable(PropTypes.bool.isRequired),
    setNumber: PropTypes.func.isRequired,
    setConsentGiven: PropTypes.func.isRequired
  }

  render () {
    const { isNumberValid, isVerified } = this.props;

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
          label={ 'phone number in international format' }
          hint={ 'the SMS will be sent to this number' }
          error={ isNumberValid ? null : 'invalid number' }
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
        <div className={ styles.terms }>{ termsOfService }</div>
      </Form>
    );
  }

  renderFee () {
    const { fee } = this.props;

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
    const { isVerified } = this.props;

    if (isVerified) {
      return (
        <div className={ styles.container }>
          <ErrorIcon />
          <p className={ styles.message }>Your account is already verified.</p>
        </div>
      );
    } else if (isVerified === false) {
      return (
        <div className={ styles.container }>
          <SuccessIcon />
          <p className={ styles.message }>Your account is not verified yet.</p>
        </div>
      );
    }
    return (
      <p className={ styles.message }>Checking if your account is verified…</p>
    );
  }

  renderRequested () {
    const { isVerified, hasRequested } = this.props;

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
    } else if (hasRequested === false) {
      return (
        <div className={ styles.container }>
          <SuccessIcon />
          <p className={ styles.message }>You did not request verification yet.</p>
        </div>
      );
    }
    return (
      <p className={ styles.message }>Checking if you requested verification…</p>
    );
  }

  numberOnSubmit = (value) => {
    this.props.setNumber(value);
  }

  numberOnChange = (_, value) => {
    this.props.setNumber(value);
  }

  consentOnChange = (_, consentGiven) => {
    this.props.setConsentGiven(consentGiven);
  }
}
