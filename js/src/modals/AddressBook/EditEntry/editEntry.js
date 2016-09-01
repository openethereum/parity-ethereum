import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Modal, Form, Input, InputAddress } from '../../../ui';
import { validateName } from '../../../services/validation';

import styles from '../style.css';

export default class EditEntry extends Component {
  static propTypes = {
    contact: PropTypes.object,
    onClose: PropTypes.func
  };

  state = {
    address: this.props.contact.address,
    name: this.props.contact.name,
    nameError: null,
    description: this.props.contact.description
  };

  render () {
    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }>
        <div className={ styles.header }>
          <h3>edit contact</h3>
        </div>
        { this.renderFields() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { nameError } = this.state;
    const hasErrors = !!(nameError);

    return ([
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />,
      <FlatButton
        disabled={ hasErrors }
        icon={ <ContentAdd /> }
        label='Save Entry'
        primary
        onTouchTap={ this.onSave } />
    ]);
  }

  renderFields () {
    const { name, nameError, description, address } = this.state;

    return (
      <Form>
        <InputAddress
          disabled
          label='contact address'
          hint='the network address for the contact'
          value={ address } />
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

  onSave = () => {
    const { address, name, description } = this.state;

    this.props.onClose(address, name, description);
  }

  onClose = () => {
    this.props.onClose();
  }
}
