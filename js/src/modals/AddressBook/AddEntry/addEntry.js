import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';

import Api from '../../../api';
import IdentityIcon from '../../../ui/IdentityIcon';
import Modal from '../../../ui/Modal';
import Form, { Input } from '../../../ui/Form';

import { ERRORS } from '../errors';

import styles from '../style.css';

export default class AddEntry extends Component {
  static propTypes = {
    onClose: PropTypes.func
  };

  state = {
    address: '',
    addressError: ERRORS.invalidAddress,
    name: '',
    nameError: ERRORS.invalidName,
    description: ''
  };

  render () {
    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }>
        <div className={ styles.header }>
          <h3>add contact</h3>
        </div>
        { this.renderFields() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { addressError, nameError } = this.state;
    const hasError = !!(addressError || nameError);

    return ([
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />,
      <FlatButton
        icon={ <ContentAdd /> }
        label='Save Entry'
        disabled={ hasError }
        primary
        onTouchTap={ this.onAdd } />
    ]);
  }

  renderFields () {
    return (
      <Form>
        <Input
          label='contact name'
          hint='a descriptive name for the contact'
          error={ this.state.nameError }
          value={ this.state.name }
          onChange={ this.onEditName } />
        <Input
          className={ styles.input }
          label='contact address'
          hint='the network address for the contact'
          error={ this.state.addressError }
          value={ this.state.address }
          onChange={ this.onEditAddress } />
        <Input
          multiLine
          rows={ 2 }
          label='(optional) contact description'
          hint='a expanded description for the contact'
          value={ this.state.description }
          onChange={ this.onEditDescription } />
      </Form>
    );
  }

  renderAddressIcon () {
    const { address, addressError } = this.state;

    if (addressError) {
      return null;
    }

    return (
      <div className={ styles.addricon }>
        <IdentityIcon
          inline center
          address={ address } />
      </div>
    );
  }

  onEditAddress = (event, address) => {
    let addressError = null;

    if (!address) {
      addressError = ERRORS.invalidAddress;
    } else if (!Api.format.isAddressValid(address)) {
      addressError = ERRORS.invalidAddress;
    } else {
      address = Api.format.toChecksumAddress(address);
    }

    this.setState({
      address,
      addressError
    });
  }

  onEditDescription = (event, description) => {
    this.setState({
      description
    });
  }

  onEditName = (event, name) => {
    const nameError = !name || name.length < 2 ? ERRORS.invalidName : null;

    this.setState({
      name,
      nameError
    });
  }

  onAdd = () => {
    const { address, name, description } = this.state;

    this.props.onClose(address, name, description);
  }

  onClose = () => {
    this.props.onClose();
  }
}
