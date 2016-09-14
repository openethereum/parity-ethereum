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
import RaisedButton from 'material-ui/RaisedButton';
import ReactTooltip from 'react-tooltip';

import { Form, Input, SignerIcon } from '../../../../../ui';

import styles from './TransactionPendingFormConfirm.css';

export default class TransactionPendingFormConfirm extends Component {

  static propTypes = {
    isSending: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired
  }

  id = Math.random(); // for tooltip

  state = {
    password: ''
  }

  render () {
    const { isSending } = this.props;
    const { password } = this.state;

    return (
      <div className={ styles.confirmForm }>
        <Form>
          <Input
            onChange={ this.onModifyPassword }
            onKeyDown={ this.onKeyDown }
            label='Account Password'
            hint='unlock the account'
            type='password'
            value={ password } />
          <div
            data-tip
            data-place='bottom'
            data-for={ 'transactionConfirmForm' + this.id }
            data-effect='solid'
          >
            <RaisedButton
              onClick={ this.onConfirm }
              className={ styles.confirmButton }
              fullWidth
              primary
              disabled={ isSending }
              icon={ <SignerIcon className={ styles.signerIcon } /> }
              label={ isSending ? 'Confirming...' : 'Confirm Transaction' }
            />
          </div>
          { this.renderTooltip() }
        </Form>
      </div>
    );
  }

  renderTooltip () {
    if (this.state.password.length) {
      return;
    }

    return (
      <ReactTooltip id={ 'transactionConfirmForm' + this.id }>
        Please provide a password for this account
      </ReactTooltip>
    );
  }

  onModifyPassword = evt => {
    const password = evt.target.value;
    this.setState({
      password
    });
  }

  onConfirm = () => {
    const { password } = this.state;
    this.props.onConfirm(password);
  }

  onKeyDown = evt => {
    if (evt.which !== 13) {
      return;
    }

    this.onConfirm();
  }
}
