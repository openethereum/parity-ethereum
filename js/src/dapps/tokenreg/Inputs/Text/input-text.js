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
          onChange={ this.onChange } />

        { this.renderLoading() }
        { this.renderIsValid() }
      </div>
    );
  }

  renderLoading () {
    if (!this.state.loading) return;

    return (
      <div className={ styles['input-loading'] }>
        <Loading size={ 0.3 } />
      </div>
    );
  }

  renderIsValid () {
    if (this.state.loading || !this.state.valid) return;

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

    let validation = validate(value, validationType, contract);

    if (validation instanceof Promise) {
      this.setState({ disabled: true, loading: true });

      return validation
        .then(validation => {
          this.setValidation({
            ...validation,
            disabled: false,
            loading: false
          });

          event.target.focus();
        });
    }

    this.setValidation(validation);
  }

  setValidation = (validation) => {
    const { value } = validation;

    this.setState({ ...validation });

    if (validation.valid) return this.props.onChange(true, value);
    return this.props.onChange(false, value);
  }

}
