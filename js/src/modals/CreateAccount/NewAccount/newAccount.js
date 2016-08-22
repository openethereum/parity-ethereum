import React, { Component, PropTypes } from 'react';

import IconButton from 'material-ui/IconButton';
import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';
import ActionAutorenew from 'material-ui/svg-icons/action/autorenew';

import Form, { Input } from '../../../ui/Form';
import IdentityIcon from '../../../ui/IdentityIcon';

import styles from '../style.css';

const ERRORS = {
  noName: 'you need to specify a valid name for the account',
  invalidPassword: 'you need to specify a password >= 8 characters',
  noMatchPassword: 'the supplied passwords does not match'
};

export default class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    errorHandler: PropTypes.func.isRequired
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  state = {
    accountName: '',
    accountNameError: ERRORS.noName,
    password1: '',
    password1Error: ERRORS.invalidPassword,
    password2: '',
    password2Error: ERRORS.noMatchPassword,
    accounts: null,
    selectedAddress: '',
    isValidPass: false,
    isValidName: false
  }

  componentWillMount () {
    this.createIdentities();
    this.props.onChange(false, {});
  }

  render () {
    return (
      <Form>
        <Input
          label='account name'
          hint='a descriptive name for the account'
          error={ this.state.accountNameError }
          value={ this.state.accountName }
          onChange={ this.onEditAccountName } />
        <div className={ styles.passwords }>
          <div className={ styles.password }>
            <Input
              className={ styles.password }
              label='password'
              hint='a strong, unique password'
              type='password'
              error={ this.state.password1Error }
              value={ this.state.password1 }
              onChange={ this.onEditPassword1 } />
          </div>
          <div className={ styles.password }>
            <Input
              className={ styles.password }
              label='password (repeat)'
              hint='verify your password'
              type='password'
              error={ this.state.password2Error }
              value={ this.state.password2 }
              onChange={ this.onEditPassword2 } />
          </div>
        </div>
        { this.renderIdentitySelector() }
        { this.renderIdentities() }
      </Form>
    );
  }

  renderIdentitySelector () {
    if (!this.state.accounts) {
      return null;
    }

    const buttons = Object.keys(this.state.accounts).map((address) => {
      return (
        <RadioButton
          className={ styles.button }
          key={ address }
          value={ address } />
      );
    });

    return (
      <RadioButtonGroup
        valueSelected={ this.state.selectedAddress }
        className={ styles.selector }
        name='identitySelector'
        onChange={ this.onChangeIdentity }>
        { buttons }
      </RadioButtonGroup>
    );
  }

  renderIdentities () {
    if (!this.state.accounts) {
      return null;
    }

    const identities = Object.keys(this.state.accounts).map((address) => {
      return (
        <div
          className={ styles.identity }
          key={ address }
          onTouchTap={ this.onChangeIdentity }>
          <IdentityIcon
            address={ address }
            center />
        </div>
      );
    });

    return (
      <div className={ styles.identities }>
        { identities }
        <div className={ styles.refresh }>
          <IconButton
            onTouchTap={ this.createIdentities }>
            <ActionAutorenew
              color='rgb(0, 151, 167)' />
          </IconButton>
        </div>
      </div>
    );
  }

  createIdentities = () => {
    const api = this.context.api;

    Promise
      .all([
        api.ethcore.generateSecretPhrase(),
        api.ethcore.generateSecretPhrase(),
        api.ethcore.generateSecretPhrase(),
        api.ethcore.generateSecretPhrase(),
        api.ethcore.generateSecretPhrase()
      ])
      .then((phrases) => {
        return Promise
          .all(phrases.map((phrase) => api.ethcore.phraseToAddress(phrase)))
          .then((addresses) => {
            const accounts = {};

            phrases.forEach((phrase, idx) => {
              accounts[addresses[idx]] = {
                address: addresses[idx],
                phrase: phrase
              };
            });

            console.log(accounts);

            this.setState({
              selectedAddress: addresses[0],
              accounts: accounts
            });
          });
      })
      .catch((error) => {
        setTimeout(this.createIdentities, 1000);
        this.context.errorHandler(error);
      });
  }

  updateParent = () => {
    this.props.onChange(this.state.isValidName && this.state.isValidPass, {
      address: this.state.selectedAddress,
      name: this.state.accountName,
      password: this.state.password1,
      phrase: this.state.accounts[this.state.selectedAddress].phrase
    });
  }

  onChangeIdentity = (event) => {
    const address = event.target.value || event.target.getAttribute('value');

    if (!address) {
      return;
    }

    this.setState({
      selectedAddress: address
    }, this.updateParent);
  }

  onEditAccountName = (event) => {
    const value = event.target.value;
    let error = null;

    if (!value || value.trim().length < 2) {
      error = ERRORS.noName;
    }

    this.setState({
      accountName: value,
      accountNameError: error,
      isValidName: !error
    }, this.updateParent);
  }

  onEditPassword1 = (event) => {
    const value = event.target.value;
    let error1 = null;
    let error2 = null;

    if (!value || value.trim().length < 8) {
      error1 = ERRORS.invalidPassword;
    }

    if (value !== this.state.password2) {
      error2 = ERRORS.noMatchPassword;
    }

    this.setState({
      password1: value,
      password1Error: error1,
      password2Error: error2,
      isValidPass: !error1 && !error2
    }, this.updateParent);
  }

  onEditPassword2 = (event) => {
    const value = event.target.value;
    let error2 = null;

    if (value !== this.state.password1) {
      error2 = ERRORS.noMatchPassword;
    }

    this.setState({
      password2: value,
      password2Error: error2,
      isValidPass: !error2
    }, this.updateParent);
  }
}
