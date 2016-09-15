import React, { Component, PropTypes } from 'react';

import { Dialog, FlatButton, TextField } from 'material-ui';

import AccountSelector from '../../Accounts/AccountSelector';

import { ADDRESS_TYPE, TLA_TYPE, UINT_TYPE, STRING_TYPE, validate } from '../validation';

import styles from '../actions.css';

const defaultField = { error: null, value: '', valid: false };
const initState = {
    fields: {
      address: { ...defaultField, type: ADDRESS_TYPE },
      tla: { ...defaultField, type: TLA_TYPE },
      base: { ...defaultField, type: UINT_TYPE },
      name: { ...defaultField, type: STRING_TYPE }
    }
  };

export default class ActionTransfer extends Component {

  static propTypes = {
    show: PropTypes.bool,
    sending: PropTypes.bool,
    complete: PropTypes.bool,
    error: PropTypes.object,
    onClose: PropTypes.func,
    handleRegisterToken: PropTypes.func
  }

  state = initState;

  render () {
    const { sending, error, complete } = this.props;

    return (
      <Dialog
        title={ error ? 'error' : 'register a new token' }
        open={ this.props.show }
        modal={ sending || complete }
        className={ styles.dialog }
        onRequestClose={ this.onClose.bind(this) }
        actions={ this.renderActions() } >
        { this.renderContent() }
      </Dialog>
    );
  }

  renderActions () {
    const { fields } = this.state;
    const { complete, sending, error } = this.props;

    if (error) {
      return (
        <FlatButton
          label='Close'
          primary
          onTouchTap={ this.onClose.bind(this) } />
      );
    }

    if (complete) {
      return (
        <FlatButton
          label='Done'
          primary
          onTouchTap={ this.onClose.bind(this) } />
      );
    }

    const isValid = this.isValid();

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.onClose.bind(this) } />,
      <FlatButton
        label={ sending ? 'Sending...' : 'Register' }
        primary
        disabled={ !isValid || sending }
        onTouchTap={ this.onRegister.bind(this) } />
    ]);
  }

  renderContent () {
    let { error, complete } = this.props;

    if (error) return this.renderError();
    if (complete) return this.renderComplete();
    return this.renderForm();
  }

  renderError () {
    let { error } = this.props;

    return (<div>
      <p className={ styles.error }>{ error.toString() }</p>
    </div>);
  }

  renderComplete () {
    return (<div>
      <p>Your transaction has been posted. Please visit the Parity Signer to authenticate the transfer.</p>
    </div>);
  }

  renderForm () {
    const { fields } = this.state;

    return (
      <div>
        <AccountSelector />

        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='Token address'
          fullWidth
          hintText='The token address'
          errorText={ fields.address.error }
          onChange={ this.onChange.bind(this, 'address') } />

        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='Token TLA'
          fullWidth
          hintText='The token short name (3 characters)'
          errorText={ fields.tla.error }
          onChange={ this.onChange.bind(this, 'tla') } />

        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='Token Base'
          fullWidth
          hintText='The token precision'
          errorText={ fields.base.error }
          onChange={ this.onChange.bind(this, 'base') } />

        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='Token name'
          fullWidth
          hintText='The token name'
          errorText={ fields.name.error }
          onChange={ this.onChange.bind(this, 'name') } />
      </div>
    );
  }

  isValid() {
    const { fields } = this.state;

    return Object.keys(fields)
      .map(key => fields[key].valid)
      .reduce((current, fieldValid) => {
        return current && fieldValid;
      }, true);
  }

  onRegister() {
    let { fields } = this.state;

    let data = Object.keys(fields)
      .reduce((dataObject, fieldKey) => {
        dataObject[fieldKey] = fields[fieldKey].value;
        return dataObject;
      }, {});

    this.props.handleRegisterToken(data);
  }

  onClose() {
    this.setState(initState);
    this.props.onClose();
  }

  onChange(fieldKey, event) {
    const value = event.target.value;

    let fields = this.state.fields;
    let fieldState = fields[fieldKey];
    let validation = validate(value, fieldState.type);

    let newFieldState = {
      ...fieldState,
      ...validation
    };

    newFieldState.value = (validation.value !== undefined)
      ? validation.value
      : value;

    this.setState({
      fields: {
        ...fields,
        [fieldKey]: newFieldState
      }
    });
  }

}
