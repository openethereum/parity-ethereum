import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Modal, Form, Input, InputAddress } from '../../../ui';
import { ERRORS, validateAddress, validateName } from '../../../services/validation';

import styles from '../style.css';

export default class AddEntry extends Component {
  static contextTypes = {
    contacts: PropTypes.array.isRequired
  };

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
    const { address, addressError, description, name, nameError } = this.state;

    return (
      <Form>
        <InputAddress
          label='contact address'
          hint='the network address for the contact'
          error={ addressError }
          value={ address }
          onChange={ this.onEditAddress } />
        <Input
          label='contact name'
          hint='a descriptive name for the contact'
          error={ nameError }
          value={ name }
          onChange={ this.onEditName } />
        <Input
          multiLine
          rows={ 2 }
          label='(optional) contact description'
          hint='a expanded description for the contact'
          value={ description }
          onChange={ this.onEditDescription } />
      </Form>
    );
  }

  onEditAddress = (event, _address) => {
    const { contacts } = this.context;
    let { address, addressError } = validateAddress(_address);

    if (!addressError) {
      const contact = contacts.find((contact) => contact.address === address);

      if (contact) {
        addressError = ERRORS.duplicateAddress;
      }
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

  onEditName = (event, _name) => {
    const { name, nameError } = validateName(_name);

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
