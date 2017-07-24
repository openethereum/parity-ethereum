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
import { FormattedMessage } from 'react-intl';
import BigNumber from 'bignumber.js';

import { fromWei } from '@parity/api/util/wei';
import { Checkbox, Form, Input } from '@parity/ui';
import { DoneIcon, ErrorIcon, InfoIcon } from '@parity/ui/Icons';
import { nullableProptype } from '@parity/shared/util/proptypes';

import smsTermsOfService from '../sms-verification/terms-of-service';
import emailTermsOfService from '../email-verification/terms-of-service';
import { howSMSVerificationWorks, howEmailVerificationWorks } from '../how-it-works';

import styles from './gatherData.css';

const boolOfError = PropTypes.oneOfType([ PropTypes.bool, PropTypes.instanceOf(Error) ]);

export default class GatherData extends Component {
  static propTypes = {
    fee: React.PropTypes.instanceOf(BigNumber),
    fields: PropTypes.array.isRequired,
    accountHasRequested: nullableProptype(PropTypes.bool.isRequired),
    isServerRunning: nullableProptype(PropTypes.bool.isRequired),
    isAbleToRequest: nullableProptype(boolOfError.isRequired),
    accountIsVerified: nullableProptype(PropTypes.bool.isRequired),
    method: PropTypes.string.isRequired,
    setConsentGiven: PropTypes.func.isRequired
  }

  render () {
    const { method, accountIsVerified } = this.props;
    const termsOfService = method === 'email' ? emailTermsOfService : smsTermsOfService;
    const howItWorks = method === 'email' ? howEmailVerificationWorks : howSMSVerificationWorks;

    return (
      <Form>
        { howItWorks }
        { this.renderServerRunning() }
        { this.renderFee() }
        { this.renderCertified() }
        { this.renderRequested() }
        { this.renderFields() }
        { this.renderIfAbleToRequest() }
        <Checkbox
          className={ styles.spacing }
          label={
            <FormattedMessage
              id='ui.verification.gatherData.termsOfService'
              defaultMessage='I agree to the terms and conditions below.'
            />
          }
          disabled={ accountIsVerified }
          onClick={ this.consentOnChange }
        />
        <div className={ styles.terms }>{ termsOfService }</div>
      </Form>
    );
  }

  renderServerRunning () {
    const { isServerRunning } = this.props;

    if (isServerRunning) {
      return (
        <div className={ styles.container }>
          <DoneIcon />
          <p className={ styles.message }>
            <FormattedMessage
              id='ui.verification.gatherData.isServerRunning.true'
              defaultMessage='The verification server is running.'
            />
          </p>
        </div>
      );
    } else if (isServerRunning === false) {
      return (
        <div className={ styles.container }>
          <ErrorIcon />
          <p className={ styles.message }>
            <FormattedMessage
              id='ui.verification.gatherData.isServerRunning.false'
              defaultMessage='The verification server is not running.'
            />
          </p>
        </div>
      );
    }
    return (
      <p className={ styles.message }>
        <FormattedMessage
          id='ui.verification.gatherData.isServerRunning.pending'
          defaultMessage='Checking if the verification server is running…'
        />
      </p>
    );
  }

  renderFee () {
    const { fee } = this.props;

    if (!fee) {
      return (<p>Fetching the fee…</p>);
    }
    if (fee.eq(0)) {
      return (
        <div className={ styles.container }>
          <InfoIcon />
          <p className={ styles.message }>
            <FormattedMessage
              id='ui.verification.gatherData.nofee'
              defaultMessage='There is no additional fee.'
            />
          </p>
        </div>
      );
    }
    return (
      <div className={ styles.container }>
        <InfoIcon />
        <p className={ styles.message }>
          <FormattedMessage
            id='ui.verification.gatherData.fee'
            defaultMessage='The additional fee is {amount} ETH.'
            values={ {
              amount: fromWei(fee).toFixed(3)
            } }
          />
        </p>
      </div>
    );
  }

  renderCertified () {
    const { accountIsVerified } = this.props;

    if (accountIsVerified) {
      return (
        <div className={ styles.container }>
          <ErrorIcon />
          <p className={ styles.message }>
            <FormattedMessage
              id='ui.verification.gatherData.accountIsVerified.true'
              defaultMessage='Your account is already verified.'
            />
          </p>
        </div>
      );
    } else if (accountIsVerified === false) {
      return (
        <div className={ styles.container }>
          <DoneIcon />
          <p className={ styles.message }>
            <FormattedMessage
              id='ui.verification.gatherData.accountIsVerified.false'
              defaultMessage='Your account is not verified yet.'
            />
          </p>
        </div>
      );
    }
    return (
      <p className={ styles.message }>
        <FormattedMessage
          id='ui.verification.gatherData.accountIsVerified.pending'
          defaultMessage='Checking if your account is verified…'
        />
      </p>
    );
  }

  renderRequested () {
    const { accountIsVerified, accountHasRequested } = this.props;

    // If the account is verified, don't show that it has requested verification.
    if (accountIsVerified) {
      return null;
    }

    if (accountHasRequested) {
      return (
        <div className={ styles.container }>
          <InfoIcon />
          <p className={ styles.message }>
            <FormattedMessage
              id='ui.verification.gatherData.accountHasRequested.true'
              defaultMessage='You already requested verification from this account.'
            />
          </p>
        </div>
      );
    } else if (accountHasRequested === false) {
      return (
        <div className={ styles.container }>
          <DoneIcon />
          <p className={ styles.message }>
            <FormattedMessage
              id='ui.verification.gatherData.accountHasRequested.false'
              defaultMessage='You did not request verification from this account yet.'
            />
          </p>
        </div>
      );
    }
    return (
      <p className={ styles.message }>
        <FormattedMessage
          id='ui.verification.gatherData.accountHasRequested.pending'
          defaultMessage='Checking if you requested verification…'
        />
      </p>
    );
  }

  renderFields () {
    const { accountIsVerified, fields } = this.props;

    const rendered = fields.map((field, index) => {
      const onChange = (_, v) => {
        field.onChange(v);
      };
      const onSubmit = field.onChange;

      return (
        <Input
          autoFocus={ index === 0 }
          className={ styles.field }
          key={ field.key }
          label={ field.label }
          hint={ field.hint }
          error={ field.error }
          disabled={ accountIsVerified }
          onChange={ onChange }
          onSubmit={ onSubmit }
        />
      );
    });

    return (<div>{rendered}</div>);
  }

  renderIfAbleToRequest () {
    const { accountIsVerified, isAbleToRequest } = this.props;

    // If the account is verified, don't show a warning.
    // If the client is able to send the request, don't show a warning
    if (accountIsVerified || isAbleToRequest === true) {
      return null;
    }

    if (isAbleToRequest === null) {
      return (
        <p className={ styles.message }>
          <FormattedMessage
            id='ui.verification.gatherData.isAbleToRequest.pending'
            defaultMessage='Validating your input…'
          />
        </p>
      );
    } else if (isAbleToRequest) {
      return (
        <div className={ styles.container }>
          <ErrorIcon />
          <p className={ styles.message }>
            { isAbleToRequest.message }
          </p>
        </div>
      );
    }
  }

  consentOnChange = (_, consentGiven) => {
    this.props.setConsentGiven(consentGiven);
  }
}
