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

import keycode from 'keycode';
import RaisedButton from 'material-ui/RaisedButton';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';
import { FormattedMessage } from 'react-intl';
import ReactTooltip from 'react-tooltip';

import { Form, Input, IdentityIcon } from '~/ui';

import styles from './transactionPendingFormConfirm.css';

export default class TransactionPendingFormConfirm extends Component {
  static propTypes = {
    account: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    disabled: PropTypes.bool,
    isSending: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired,
    focus: PropTypes.bool
  };

  static defaultProps = {
    focus: false
  };

  id = Math.random(); // for tooltip

  state = {
    password: '',
    wallet: null,
    walletError: null
  }

  componentDidMount () {
    this.focus();
  }

  componentWillReceiveProps (nextProps) {
    if (!this.props.focus && nextProps.focus) {
      this.focus(nextProps);
    }
  }

  /**
   * Properly focus on the input element when needed.
   * This might be fixed some day in MaterialUI with
   * an autoFocus prop.
   *
   * @see https://github.com/callemall/material-ui/issues/5632
   */
  focus (props = this.props) {
    if (props.focus) {
      const textNode = ReactDOM.findDOMNode(this.refs.input);

      if (!textNode) {
        return;
      }

      const inputNode = textNode.querySelector('input');

      inputNode && inputNode.focus();
    }
  }

  getPasswordHint () {
    const { account } = this.props;
    const accountHint = account && account.meta && account.meta.passwordHint;

    if (accountHint) {
      return accountHint;
    }

    const { wallet } = this.state;
    const walletHint = wallet && wallet.meta && wallet.meta.passwordHint;

    return walletHint || null;
  }

  render () {
    const { account, address, disabled, isSending } = this.props;
    const { wallet, walletError } = this.state;
    const isWalletOk = account.hardware || account.uuid || (walletError === null && wallet !== null);

    return (
      <div className={ styles.confirmForm }>
        <Form>
          { this.renderKeyInput() }
          { this.renderPassword() }
          { this.renderHint() }
          <div
            data-effect='solid'
            data-for={ `transactionConfirmForm${this.id}` }
            data-place='bottom'
            data-tip
          >
            <RaisedButton
              className={ styles.confirmButton }
              disabled={ disabled || isSending || !isWalletOk }
              fullWidth
              icon={
                <IdentityIcon
                  address={ address }
                  button
                  className={ styles.signerIcon }
                />
              }
              label={
                isSending
                  ? (
                    <FormattedMessage
                      id='signer.txPendingConfirm.buttons.confirmBusy'
                      defaultMessage='Confirming...'
                    />
                  )
                  : (
                    <FormattedMessage
                      id='signer.txPendingConfirm.buttons.confirmRequest'
                      defaultMessage='Confirm Request'
                    />
                  )
              }
              onTouchTap={ this.onConfirm }
              primary
            />
          </div>
          { this.renderTooltip() }
        </Form>
      </div>
    );
  }

  renderPassword () {
    const { account } = this.props;
    const { password } = this.state;

    if (account && account.hardware) {
      return null;
    }

    return (
      <Input
        hint={
          account.uuid
            ? (
              <FormattedMessage
                id='signer.txPendingConfirm.password.unlock.hint'
                defaultMessage='unlock the account'
              />
            )
            : (
              <FormattedMessage
                id='signer.txPendingConfirm.password.decrypt.hint'
                defaultMessage='decrypt the key'
              />
            )
        }
        label={
          account.uuid
            ? (
              <FormattedMessage
                id='signer.txPendingConfirm.password.unlock.label'
                defaultMessage='Account Password'
              />
            )
            : (
              <FormattedMessage
                id='signer.txPendingConfirm.password.decrypt.label'
                defaultMessage='Key Password'
              />
            )
        }
        onChange={ this.onModifyPassword }
        onKeyDown={ this.onKeyDown }
        ref='input'
        type='password'
        value={ password }
      />
    );
  }

  renderHint () {
    const { account, disabled, isSending } = this.props;

    if (account.hardware) {
      if (isSending) {
        return (
          <div className={ styles.passwordHint }>
            <FormattedMessage
              id='signer.sending.hardware.confirm'
              defaultMessage='Please confirm the transaction on your attached hardware device'
            />
          </div>
        );
      } else if (disabled) {
        return (
          <div className={ styles.passwordHint }>
            <FormattedMessage
              id='signer.sending.hardware.connect'
              defaultMessage='Please attach your hardware device before confirming the transaction'
            />
          </div>
        );
      }
    }

    const passwordHint = this.getPasswordHint();

    if (!passwordHint) {
      return null;
    }

    return (
      <div className={ styles.passwordHint }>
        <FormattedMessage
          id='signer.txPendingConfirm.passwordHint'
          defaultMessage='(hint) {passwordHint}'
          values={ {
            passwordHint
          } }
        />
      </div>
    );
  }

  renderKeyInput () {
    const { account } = this.props;
    const { walletError } = this.state;

    if (account.uuid || account.wallet || account.hardware) {
      return null;
    }

    return (
      <Input
        className={ styles.fileInput }
        error={ walletError }
        hint={
          <FormattedMessage
            id='signer.txPendingConfirm.selectKey.hint'
            defaultMessage='The keyfile to use for this account'
          />
        }
        label={
          <FormattedMessage
            id='signer.txPendingConfirm.selectKey.label'
            defaultMessage='Select Local Key'
          />
        }
        onChange={ this.onKeySelect }
        type='file'
      />
    );
  }

  renderTooltip () {
    const { account } = this.props;

    if (this.state.password.length || account.hardware) {
      return;
    }

    return (
      <ReactTooltip id={ `transactionConfirmForm${this.id}` }>
        <FormattedMessage
          id='signer.txPendingConfirm.tooltips.password'
          defaultMessage='Please provide a password for this account'
        />
      </ReactTooltip>
    );
  }

  onKeySelect = (event) => {
    // Check that file have been selected
    if (event.target.files.length === 0) {
      return this.setState({
        wallet: null,
        walletError: null
      });
    }

    const fileReader = new FileReader();

    fileReader.onload = (e) => {
      try {
        const wallet = JSON.parse(e.target.result);

        try {
          if (wallet && typeof wallet.meta === 'string') {
            wallet.meta = JSON.parse(wallet.meta);
          }
        } catch (e) {}

        this.setState({
          wallet,
          walletError: null
        });
      } catch (error) {
        this.setState({
          wallet: null,
          walletError: (
            <FormattedMessage
              id='signer.txPendingConfirm.errors.invalidWallet'
              defaultMessage='Given wallet file is invalid.'
            />
          )
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
      password,
      wallet
    });
  }

  onKeyDown = (event) => {
    const codeName = keycode(event);

    if (codeName !== 'enter') {
      return;
    }

    this.onConfirm();
  }
}
