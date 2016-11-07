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
import phone from 'phoneformat.js';

import { Form, Input } from '../../../ui';

import styles from './gatherData.css';

export default class GatherData extends Component {
  static propTypes = {
    onDataIsValid: PropTypes.func.isRequired,
    onDataIsInvalid: PropTypes.func.isRequired,
    onData: PropTypes.func.isRequired
  }

  state = {
    numberIsValid: null,
    consentGiven: false
  };

  render () {
    const { numberIsValid } = this.state;

    return (
      <Form>
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
    const { numberIsValid, consentGiven } = this.state;

    if (numberIsValid && consentGiven) {
      this.props.onDataIsValid();
    } else {
      this.props.onDataIsInvalid();
    }
  }
}
