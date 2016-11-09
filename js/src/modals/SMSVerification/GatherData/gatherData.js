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
import SuccessIcon from 'material-ui/svg-icons/navigation/check';
import ErrorIcon from 'material-ui/svg-icons/alert/error-outline';

import phone from 'phoneformat.js';

import { fromWei } from '../../../api/util/wei';
import { Form, Input } from '../../../ui';

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
    isCertified: null,
    numberIsValid: null,
    consentGiven: false
  };

  componentWillMount () {
    const { init } = this.state;
    if (init) {
      this.setState({ init: false });
      this.queryFee();
      this.checkIfCertified();
    }
  }

  render () {
    const { fee } = this.props.data;
    const { numberIsValid } = this.state;

    return (
      <Form>
        <p>{ fee ? `The fee is ${fromWei(fee).toFixed(3)} ETH.` : 'Fetching the fee…' }</p>
        { this.renderCertified() }
        <Input
          label={ 'phone number' }
          hint={ 'the sms will be sent to this number' }
          error={ numberIsValid ? null : 'invalid number' }
          onChange={ this.numberOnChange }
          onSubmit={ this.numberOnSubmit }
        />
        <Checkbox
          className={ styles.spacing }
          label={ 'I agree that my number will be stored.' }
          onCheck={ this.consentOnChange }
        />
      </Form>
    );
  }

  renderCertified () {
    const { isCertified } = this.props.data;

    if (isCertified) {
      return (
        <div className={ styles.container }>
          <ErrorIcon />
          <p className={ styles.message }>Your account is already verified.</p>
        </div>
      );
    }
    if (isCertified === false) {
      return (
        <div className={ styles.container }>
          <SuccessIcon />
          <p className={ styles.message }>Your account is not verified yet.</p>
        </div>
      );
    }
    return (<p className={ styles.message }>Checking if your account is verified…</p>);
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

    contract.instance.certified.call({}, [account])
    .then((isCertified) => {
      onData({ isCertified });
      this.onChange();
    })
    .catch((err) => {
      console.error('error checking if certified', err);
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
    const { fee, isCertified } = this.props.data;
    const { numberIsValid, consentGiven } = this.state;

    if (fee && numberIsValid && consentGiven && isCertified === false) {
      this.props.onDataIsValid();
    } else {
      this.props.onDataIsInvalid();
    }
  }
}
