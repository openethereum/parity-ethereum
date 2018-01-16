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

import { Form, Input, IdentityIcon, QrCode, QrScan } from '~/ui';
import { generateTxQr, generateDataQr, generateDecryptQr } from '~/util/qrscan';

import styles from './transactionPendingFormConfirm.css';

const QR_VISIBLE = 1;
const QR_SCAN = 2;
const QR_COMPLETED = 3;

export default class TransactionPendingFormConfirm extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    account: PropTypes.object,
    address: PropTypes.string.isRequired,
    disabled: PropTypes.bool,
    focus: PropTypes.bool,
    netVersion: PropTypes.string.isRequired,
    isSending: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired,
    dataToSign: PropTypes.object.isRequired
  };

  static defaultProps = {
    account: {},
    focus: false
  };

  id = Math.random(); // for tooltip

  state = {
    password: '',
    qrState: QR_VISIBLE,
    qr: {},
    wallet: null,
    walletError: null
  }

  componentDidMount () {
    this.focus();
  }

  componentWillMount () {
    this.readNonce();
    this.subscribeNonce();
  }

  componentWillUnmount () {
    this.unsubscribeNonce();
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
    const accountHint = account.meta && account.meta.passwordHint;

    if (accountHint) {
      return accountHint;
    }

    const { wallet } = this.state;
    const walletHint = wallet && wallet.meta && wallet.meta.passwordHint;

    return walletHint || null;
  }

  // TODO: Now that we have 3 types, it would make sense splitting each into their own
  // sub-module and having the consistent bits combined (e.g. i18n, layouts)
  render () {
    const { account, address, disabled, isSending } = this.props;
    const { wallet, walletError } = this.state;
    const isAccount = account.external || account.hardware || account.uuid;
    const isWalletOk = isAccount || (walletError === null && wallet !== null);
    const confirmText = this.renderConfirmButton();
    const confirmButton = confirmText
      ? (
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
            label={ confirmText }
            onTouchTap={ this.onConfirm }
            primary
          />
        </div>
      )
      : null;

    return (
      <div className={ styles.confirmForm }>
        <Form>
          { this.renderKeyInput() }
          { this.renderQrCode() }
          { this.renderQrScanner() }
          { this.renderPassword() }
          { this.renderHint() }
          { confirmButton }
          { this.renderTooltip() }
        </Form>
      </div>
    );
  }

  renderConfirmButton () {
    const { account, isSending } = this.props;
    const { qrState } = this.state;

    if (account.external) {
      switch (qrState) {
        case QR_VISIBLE:
          return (
            <FormattedMessage
              id='signer.txPendingConfirm.buttons.scanSigned'
              defaultMessage='Scan Signed QR'
            />
          );

        case QR_SCAN:
        case QR_COMPLETED:
          return null;
      }
    }

    return isSending
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
      );
  }

  renderPassword () {
    const { account } = this.props;
    const { password } = this.state;

    if (account.hardware || account.external) {
      return null;
    }

    const isAccount = account.uuid;

    return (
      <Input
        hint={
          isAccount
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
          isAccount
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
    const { qrState } = this.state;

    if (account.external) {
      switch (qrState) {
        case QR_VISIBLE:
          return (
            <div className={ styles.passwordHint }>
              <FormattedMessage
                id='signer.sending.external.scanTx'
                defaultMessage='Please scan the transaction QR on your external device'
              />
            </div>
          );

        case QR_SCAN:
          return (
            <div className={ styles.passwordHint }>
              <FormattedMessage
                id='signer.sending.external.scanSigned'
                defaultMessage='Scan the QR code of the signed transaction from your external device'
              />
            </div>
          );

        case QR_COMPLETED:
          return null;
      }
    }

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

  // TODO: Split into sub-scomponent
  renderQrCode () {
    const { account } = this.props;
    const { qrState, qr } = this.state;

    if (!account.external || qrState !== QR_VISIBLE || !qr.value) {
      return null;
    }

    return (
      <QrCode
        className={ styles.qr }
        value={ qr.value }
      />
    );
  }

  // TODO: Split into sub-scomponent
  renderQrScanner () {
    const { account } = this.props;
    const { qrState } = this.state;

    if (!account.external || qrState !== QR_SCAN) {
      return null;
    }

    return (
      <QrScan
        className={ styles.camera }
        onScan={ this.onScan }
      />
    );
  }

  renderKeyInput () {
    const { account } = this.props;
    const { walletError } = this.state;

    if (account.uuid || account.wallet || account.hardware || account.external) {
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

    if (this.state.password.length || account.hardware || account.external) {
      return null;
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

  onScan = (signature) => {
    const { chainId, rlp, tx, data, decrypt } = this.state.qr;

    if (!signature) {
      return;
    }

    if (signature.substr(0, 2) !== '0x') {
      signature = `0x${signature}`;
    }

    this.setState({ qrState: QR_COMPLETED });

    if (tx) {
      this.props.onConfirm({
        txSigned: {
          chainId,
          rlp,
          signature,
          tx
        }
      });
      return;
    }

    if (decrypt) {
      this.props.onConfirm({
        decrypted: {
          decrypt,
          msg: signature
        }
      });
      return;
    }

    this.props.onConfirm({
      dataSigned: {
        data,
        signature
      }
    });
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
    const { account } = this.props;
    const { password, qrState, wallet } = this.state;

    if (account.external && qrState === QR_VISIBLE) {
      return this.setState({ qrState: QR_SCAN });
    }

    this.props.onConfirm({
      password,
      wallet
    });
  }

  generateQr = () => {
    const { api } = this.context;
    const { netVersion, dataToSign } = this.props;
    const { transaction, data, decrypt } = dataToSign;
    const setState = qr => {
      this.setState({ qr });
    };

    if (transaction) {
      generateTxQr(api, netVersion, transaction).then(setState);
      return;
    }

    if (decrypt) {
      generateDecryptQr(decrypt).then(setState);
      return;
    }

    generateDataQr(data).then(setState);
  }

  onKeyDown = (event) => {
    const codeName = keycode(event);

    if (codeName !== 'enter') {
      return;
    }

    this.onConfirm();
  }

  // FIXME: Sadly the API subscription channels currently does not allow for specific values,
  // rather it can only do general queries where parameters are not specified. Hence we are
  // polling for the nonce here. Since we are moving to node-based subscriptions on the API layer,
  // this can be optimised when the subscription mechanism is reworked to conform.
  subscribeNonce () {
    const nonceTimerId = setInterval(this.readNonce, 1000);

    this.setState({ nonceTimerId });
  }

  unsubscribeNonce () {
    const { nonceTimerId } = this.state;

    if (!nonceTimerId) {
      return;
    }

    clearInterval(nonceTimerId);
  }

  readNonce = () => {
    const { api } = this.context;
    const { account, dataToSign } = this.props;
    const { qr } = this.state;

    if ((dataToSign.data || dataToSign.decrypt) && qr && !qr.value) {
      this.generateQr();
      return;
    }

    if (!account || !account.external || !api.transport.isConnected || !dataToSign.transaction) {
      return;
    }

    return api.parity
      .nextNonce(account.address)
      .then((newNonce) => {
        const { nonce } = this.state.qr;

        if (!nonce || !newNonce.eq(nonce)) {
          this.generateQr();
        }
      });
  }
}
