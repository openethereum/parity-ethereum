import React, { Component, PropTypes } from 'react';

import { TextField } from 'material-ui';

import Form from '../../Form';
import FormWrap from '../../FormWrap';
import IdentityIcon from '../../IdentityIcon';

import styles from '../style.css';

export default class RecoverAccount extends Component {
  static propTypes = {
    address: PropTypes.string,
    name: PropTypes.string,
    phrase: PropTypes.string
  }

  render () {
    let info = 'The details for your newly created account is displayed below. ';
    if (this.props.phrase) {
      info += 'Take note of your recovery phrase and store it in a secure location, without it you cannot recover your account should you lose your password.';
    }

    return (
      <Form>
        <IdentityIcon
          address={ this.props.address } />
        <div className={ styles.info }>
           { info }
        </div>
        <FormWrap>
          <TextField
            autoComplete='off'
            disabled
            hintText='A descriptive name for the account'
            floatingLabelText='Account Name'
            fullWidth
            value={ this.props.name } />
        </FormWrap>
        <FormWrap>
          <TextField
            autoComplete='off'
            disabled
            hintText='The network address for the account'
            floatingLabelText='Address'
            fullWidth
            value={ this.props.address } />
        </FormWrap>
        { this.renderPhrase() }
      </Form>
    );
  }

  renderPhrase () {
    if (!this.props.phrase) {
      return null;
    }

    return (
      <FormWrap>
        <TextField
          autoComplete='off'
          disabled
          hintText='The account recovery phrase'
          floatingLabelText='Recovery Phrase'
          fullWidth
          multiLine
          rows={ 1 }
          value={ this.props.phrase } />
      </FormWrap>
    );
  }
}
