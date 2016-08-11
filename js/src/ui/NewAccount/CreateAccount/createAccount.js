import React, { Component, PropTypes } from 'react';

import { TextField } from 'material-ui';
import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';

import Form from '../../Form';
import IdentityIcon from '../../IdentityIcon';

import styles from '../style.css';

export default class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  state = {
    accountName: '',
    password1: '',
    password2: '',
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
        <div className={ styles.info }>
          Provide a descriptive name for the account, a strong password and select your preferred identity icon to create the account
        </div>
        <TextField
          autoComplete='off'
          floatingLabelText='Account Name'
          fullWidth
          hintText='A descriptive name for the account'
          value={ this.state.accountName }
          onChange={ this.onEditAccountName } />
        <div className={ styles.passwords }>
          <div className={ styles.password }>
            <TextField
              autoComplete='off'
              className={ styles.password }
              floatingLabelText='Password'
              fullWidth
              hintText='A strong, unique password'
              type='password'
              value={ this.state.password1 }
              onChange={ this.onEditPassword1 } />
          </div>
          <div className={ styles.password }>
            <TextField
              autoComplete='off'
              className={ styles.password }
              floatingLabelText='Password (repeat)'
              fullWidth
              hintText='A strong, unique password'
              type='password'
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
      </div>
    );
  }

  createIdentities () {
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
      });
  }

  passChanges = () => {
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
    }, this.passChanges);
  }

  onEditAccountName = (event) => {
    const value = event.target.value;
    const valid = value.length >= 2;

    this.setState({
      accountName: value,
      isValidName: valid
    }, this.passChanges);
  }

  onEditPassword1 = (event) => {
    const value = event.target.value;
    const valid = value.length >= 8 && this.state.password2 === value;

    this.setState({
      password1: value,
      isValidPass: valid
    }, this.passChanges);
  }

  onEditPassword2 = (event) => {
    const value = event.target.value;
    const valid = value.length >= 8 && this.state.password1 === value;

    this.setState({
      password2: value,
      isValidPass: valid
    }, this.passChanges);
  }
}
