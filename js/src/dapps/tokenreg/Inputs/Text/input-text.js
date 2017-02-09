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

import React, { Component, PropTypes } from 'react';

import { TextField } from 'material-ui';
import CheckIcon from 'material-ui/svg-icons/navigation/check';
import { green500 } from 'material-ui/styles/colors';

import Loading from '../../Loading';

import { validate } from '../validation';

import styles from '../inputs.css';

const initState = {
  error: null,
  value: '',
  valid: false,
  disabled: false,
  loading: false
};

export default class InputText extends Component {
  static propTypes = {
    validationType: PropTypes.string.isRequired,
    onChange: PropTypes.func.isRequired,
    onEnter: PropTypes.func,

    floatingLabelText: PropTypes.string,
    hintText: PropTypes.string,

    contract: PropTypes.object
  }

  state = initState;

  render () {
    const { disabled, error } = this.state;

    return (
      <div className={ styles['input-container'] }>
        <TextField
          floatingLabelText={ this.props.floatingLabelText }
          hintText={ this.props.hintText }

          autoComplete='off'
          floatingLabelFixed
          fullWidth
          disabled={ disabled }
          errorText={ error }
          onChange={ this.onChange }
          onKeyDown={ this.onKeyDown }
        />

        { this.renderLoading() }
        { this.renderIsValid() }
      </div>
    );
  }

  renderLoading () {
    if (!this.state.loading) {
      return;
    }

    return (
      <div className={ styles['input-loading'] }>
        <Loading size={ 0.3 } />
      </div>
    );
  }

  renderIsValid () {
    if (this.state.loading || !this.state.valid) {
      return;
    }

    return (
      <div className={ styles['input-icon'] }>
        <CheckIcon color={ green500 } />
      </div>
    );
  }

  onChange = (event) => {
    const value = event.target.value;

    // So we can focus on the input after async validation
    event.persist();

    const { validationType, contract } = this.props;
    const validation = validate(value, validationType, contract);

    const loadingTimeout = setTimeout(() => {
      this.setState({ disabled: true, loading: true });
    }, 50);

    return Promise.resolve(validation)
      .then((validation) => {
        clearTimeout(loadingTimeout);

        this.setValidation({
          ...validation,
          disabled: false,
          loading: false
        });

        event.target.focus();
      });
  }

  onKeyDown = (event) => {
    if (!this.props.onEnter) {
      return;
    }

    if (event.keyCode !== 13) {
      return;
    }

    this.props.onEnter();
  }

  setValidation = (validation) => {
    const { value } = validation;

    this.setState({ ...validation });

    if (validation.valid) {
      return this.props.onChange(true, value);
    }

    return this.props.onChange(false, value);
  }
}
