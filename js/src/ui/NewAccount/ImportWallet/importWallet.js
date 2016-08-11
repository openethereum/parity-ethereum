import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';

import { FlatButton, TextField } from 'material-ui';
import CommunicationImportExport from 'material-ui/svg-icons/communication/import-export';

import Form from '../../Form';

import styles from '../style.css';

export default class ImportWallet extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  state = {
    accountName: '',
    password: '',
    walletFile: '',
    walletJson: '',
    isValidPass: false,
    isValidName: false,
    isValidFile: false
  }

  componentWillMount () {
    this.props.onChange(false, {});
  }

  render () {
    return (
      <Form>
        <div className={ styles.info }>
          Provide a descriptive name for the account, the password required to unlock the account and the on-disk location of the wallet to be imported.
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
              hintText='The password to unlock the wallet'
              type='password'
              value={ this.state.password }
              onChange={ this.onEditPassword } />
          </div>
        </div>
        <TextField
          autoComplete='off'
          disabled
          floatingLabelText='Wallet file'
          fullWidth
          hintText='The uploaded file for import'
          value={ this.state.walletFile } />
        <div className={ styles.upload }>
          <FlatButton
            icon={ <CommunicationImportExport /> }
            label='Select file'
            primary
            onClick={ this.openFileDialog } />
          <input
            ref='fileUpload'
            type='file'
            style={ { display: 'none' } }
            onChange={ this.onFileChange } />
        </div>
      </Form>
    );
  }

  onFileChange = (event) => {
    const el = event.target;

    if (el.files.length) {
      const reader = new FileReader();
      reader.onload = (event) => {
        this.setState({
          walletJson: event.target.result,
          isValidFile: true
        }, this.passChanges);
      };
      reader.readAsText(el.files[0]);
    }

    this.setState({
      walletFile: el.value,
      isValidFile: false
    }, this.passChanges);
  }

  openFileDialog = () => {
    ReactDOM.findDOMNode(this.refs.fileUpload).click();
  }

  passChanges = () => {
    const valid = this.state.isValidName && this.state.isValidPass && this.state.isValidFile;

    this.props.onChange(valid, {
      name: this.state.accountName,
      password: this.state.password,
      phrase: null,
      json: this.state.walletJson
    });
  }

  onEditAccountName = (event) => {
    const value = event.target.value;
    const valid = value.length >= 2;

    this.setState({
      accountName: value,
      isValidName: valid
    }, this.passChanges);
  }

  onEditPassword = (event) => {
    const value = event.target.value;
    const valid = value.length >= 8;

    this.setState({
      password: value,
      isValidPass: valid
    }, this.passChanges);
  }
}
