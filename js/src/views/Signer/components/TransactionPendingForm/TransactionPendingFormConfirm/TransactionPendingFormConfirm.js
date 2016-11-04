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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import RaisedButton from 'material-ui/RaisedButton';
import ReactTooltip from 'react-tooltip';

import { Form, Input, IdentityIcon } from '../../../../../ui';

import styles from './TransactionPendingFormConfirm.css';

class TransactionPendingFormConfirm extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    isSending: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired
  }

  id = Math.random(); // for tooltip

  state = {
    password: ''
  }

  render () {
    const { accounts, address, isSending } = this.props;
    const { password } = this.state;
    const account = accounts[address] || {};

    const passwordHint = account.meta && account.meta.passwordHint
      ? (<div><span>(hint) </span>{ account.meta.passwordHint }</div>)
      : null;

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
          <div className={ styles.passwordHint }>
            { passwordHint }
          </div>
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
              icon={ <IdentityIcon address={ address } button className={ styles.signerIcon } /> }
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
