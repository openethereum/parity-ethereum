import React, { Component, PropTypes } from 'react';
import RaisedButton from 'material-ui/RaisedButton';
import ReactTooltip from 'react-tooltip';

import { Input, SignerIcon } from '../../../../../ui';

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
