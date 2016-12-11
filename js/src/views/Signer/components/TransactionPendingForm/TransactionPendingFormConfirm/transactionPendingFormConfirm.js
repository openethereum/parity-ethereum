// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import RaisedButton from 'material-ui/RaisedButton';
import ReactTooltip from 'react-tooltip';

import { Form, Input, IdentityIcon } from '~/ui';

import styles from './transactionPendingFormConfirm.css';

class TransactionPendingFormConfirm extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    isSending: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired
  }

  id = Math.random(); // for tooltip

  state = {
    password: '',
    wallet: null,
    walletError: null
  }

  render () {
    const { accounts, address, isSending } = this.props;
    const { password, wallet, walletError } = this.state;
    const account = accounts[address] || {};
    const isExternal = !account.uuid;

    const passwordHint = account.meta && account.meta.passwordHint
      ? (<div><span>(hint) </span>{ account.meta.passwordHint }</div>)
      : null;

    const isWalletOk = !isExternal || (walletError === null && wallet !== null);
    const keyInput = isExternal
      ? this.renderKeyInput()
      : null;

    return (
      <div className={ styles.confirmForm }>
        <Form>
          { keyInput }
          <Input
            hint={
              isExternal
                ? 'decrypt the key'
                : 'unlock the account'
            }
            label={
              isExternal
                ? 'Key Password'
                : 'Account Password'
            }
            onChange={ this.onModifyPassword }
            onKeyDown={ this.onKeyDown }
            type='password'
            value={ password } />
          <div className={ styles.passwordHint }>
            { passwordHint }
          </div>
          <div
            data-effect='solid'
            data-for={ `transactionConfirmForm${this.id}` }
            data-place='bottom'
            data-tip>
            <RaisedButton
              className={ styles.confirmButton }
              disabled={ isSending || !isWalletOk }
              fullWidth
              icon={
                <IdentityIcon
                  address={ address }
                  button
                  className={ styles.signerIcon } />
              }
              label={
                isSending
                  ? 'Confirming...'
                  : 'Confirm Transaction'
              }
              onTouchTap={ this.onConfirm }
              primary />
          </div>
          { this.renderTooltip() }
        </Form>
      </div>
    );
  }

  renderKeyInput () {
    const { walletError } = this.state;

    return (
      <Input
        className={ styles.fileInput }
        error={ walletError }
        label='Select Local Key'
        onChange={ this.onKeySelect }
        type='file' />
    );
  }

  renderTooltip () {
    if (this.state.password.length) {
      return;
    }

    return (
      <ReactTooltip id={ `transactionConfirmForm${this.id}` }>
        Please provide a password for this account
      </ReactTooltip>
    );
  }

  onKeySelect = (event) => {
    const fileReader = new FileReader();

    fileReader.onload = (e) => {
      try {
        const wallet = JSON.parse(e.target.result);

        this.setState({
          wallet,
          walletError: null
        });
      } catch (error) {
        this.setState({
          wallet: null,
          walletError: 'Given wallet file is invalid.'
        });
      }
    };

    fileReader.readAsText(event.target.files[0]);
  }

  onModifyPassword = (event) => {
    const password = event.target.value;

    this.setState({
      password
    });
  }

  onConfirm = () => {
    const { password, wallet } = this.state;

    this.props.onConfirm({
      password, wallet
    });
  }

  onKeyDown = (event) => {
    if (event.which !== 13) {
      return;
    }

    this.onConfirm();
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return {
    accounts
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TransactionPendingFormConfirm);
