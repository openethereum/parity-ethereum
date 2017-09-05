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

import React, { Component, PropTypes } from 'react';

import { Dialog, FlatButton } from 'material-ui';

import AccountSelector from '../../Accounts/AccountSelector';
import InputText from '../../Inputs/Text';

import { TOKEN_ADDRESS_TYPE, TLA_TYPE, DECIMAL_TYPE, STRING_TYPE } from '../../Inputs/validation';

import styles from '../actions.css';

const defaultField = { value: '', valid: false };
const initState = {
  isFormValid: false,
  fields: {
    address: {
      ...defaultField,
      type: TOKEN_ADDRESS_TYPE,
      floatingLabelText: 'Token address',
      hintText: 'The token address'
    },
    tla: {
      ...defaultField,
      type: TLA_TYPE,
      floatingLabelText: 'Token TLA',
      hintText: 'The token short name (3 characters)'
    },
    decimals: {
      ...defaultField,
      type: DECIMAL_TYPE,
      floatingLabelText: 'Token Decimals',
      hintText: 'The number of decimals (0-18)'
    },
    name: {
      ...defaultField,
      type: STRING_TYPE,
      floatingLabelText: 'Token name',
      hintText: 'The token name'
    }
  }
};

export default class RegisterAction extends Component {
  static propTypes = {
    show: PropTypes.bool.isRequired,
    sending: PropTypes.bool.isRequired,
    complete: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired,
    handleRegisterToken: PropTypes.func.isRequired,

    error: PropTypes.object
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
        onRequestClose={ this.onClose }
        actions={ this.renderActions() }
        ref='dialog'
        autoScrollBodyContent
      >
        { this.renderContent() }
      </Dialog>
    );
  }

  renderActions () {
    const { complete, sending, error } = this.props;

    if (error) {
      return (
        <FlatButton
          label='Close'
          primary
          onTouchTap={ this.onClose }
        />
      );
    }

    if (complete) {
      return (
        <FlatButton
          label='Done'
          primary
          onTouchTap={ this.onClose }
        />
      );
    }

    const isValid = this.state.isFormValid;

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.onClose }
      />,
      <FlatButton
        label={ sending ? 'Sending...' : 'Register' }
        primary
        disabled={ !isValid || sending }
        onTouchTap={ this.onRegister }
      />
    ]);
  }

  renderContent () {
    const { error, complete } = this.props;

    if (error) {
      return this.renderError();
    } else if (complete) {
      return this.renderComplete();
    }

    return this.renderForm();
  }

  renderError () {
    const { error } = this.props;

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
    return (
      <div>
        <AccountSelector
          onAccountChange={ this.onAccountChange }
        />
        { this.renderInputs() }
      </div>
    );
  }

  renderInputs () {
    const { fields } = this.state;

    return Object.keys(fields).map((fieldKey, index) => {
      const onChange = this.onChange.bind(this, fieldKey);
      const field = fields[fieldKey];

      return (
        <InputText
          key={ index }

          floatingLabelText={ field.floatingLabelText }
          hintText={ field.hintText }

          validationType={ field.type }
          onChange={ onChange }
        />
      );
    });
  }

  onAccountChange = () => {
    const { dialog } = this.refs;

    dialog.forceUpdate();
  }

  onChange (fieldKey, valid, value) {
    const { fields } = this.state;
    const field = fields[fieldKey];

    const newFields = {
      ...fields,
      [ fieldKey ]: {
        ...field,
        valid, value
      }
    };

    const isFormValid = Object.keys(newFields)
      .map(key => newFields[key].valid)
      .reduce((current, fieldValid) => {
        return current && fieldValid;
      }, true);

    this.setState({
      fields: newFields,
      isFormValid
    });
  }

  onRegister = () => {
    const { fields } = this.state;

    const data = Object.keys(fields)
      .reduce((dataObject, fieldKey) => {
        dataObject[fieldKey] = fields[fieldKey].value;
        return dataObject;
      }, {});

    this.props.handleRegisterToken(data);
  }

  onClose = () => {
    this.setState(initState);
    this.props.onClose();
  }
}
