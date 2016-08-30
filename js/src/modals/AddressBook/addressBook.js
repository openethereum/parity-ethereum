import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';
import ContentCreate from 'material-ui/svg-icons/content/create';

import IdentityIcon from '../../ui/IdentityIcon';
import Modal from '../../ui/Modal';

import AddEntry from './AddEntry';
import EditEntry from './EditEntry';

import styles from './style.css';

const editIconStyle = {
  color: 'rgb(0, 151, 167)',
  width: '16px',
  height: '16px'
};

export default class AddressBook extends Component {
  static contextTypes = {
    api: PropTypes.object,
    contacts: PropTypes.array
  };

  static propTypes = {
    onClose: PropTypes.func
  };

  state = {
    showAdd: false,
    showEdit: false,
    editing: null
  };

  render () {
    return (
      <Modal
        scroll visible
        actions={ this.renderDialogActions() }>
        { this.renderModals() }
        <div className={ styles.header }>
          <h3>address book entries</h3>
        </div>
        { this.renderEntries() }
      </Modal>
    );
  }

  renderDialogActions () {
    return ([
      <FlatButton
        icon={ <ContentAdd /> }
        label='Add Entry'
        primary
        onTouchTap={ this.onAdd } />,
      <FlatButton
        icon={ <ContentClear /> }
        label='Close'
        primary
        onTouchTap={ this.onClose } />
    ]);
  }

  renderEntries () {
    const { contacts } = this.context;

    if (!contacts.length) {
      return (
        <div className={ styles.noentries }>
          There are currently no address book entries.
        </div>
      );
    }

    const list = contacts.map((contact) => {
      return (
        <div
          key={ contact.address }
          className={ styles.account }>
          <IdentityIcon
            center inline
            address={ contact.address } />
          <div className={ styles.details }>
            <div
              className={ styles.name }
              onTouchTap={ this.wrapOnEdit(contact) }>
              <span>{ contact.name || 'Unnamed' }</span>
              <ContentCreate
                style={ editIconStyle }
                className={ styles.editicon } />
            </div>
            <div className={ styles.address }>
              { contact.address }
            </div>
          </div>
        </div>
      );
    });

    return (
      <div>
        { list }
      </div>
    );
  }

  renderModals () {
    const { showAdd, showEdit, editing } = this.state;

    if (showAdd) {
      return (
        <AddEntry
          onClose={ this.onCloseAdd } />
      );
    } else if (showEdit) {
      return (
        <EditEntry
          contact={ editing }
          onClose={ this.onCloseEdit } />
      );
    }

    return null;
  }

  onAdd = () => {
    this.setState({
      showAdd: true
    });
  }

  updateDetails (address, name, description) {
    const { api } = this.context;

    Promise.all(
      api.personal.setAccountName(address, name),
      api.personal.setAccountMeta(address, {
        description: description || null
      })
    );
  }

  onCloseAdd = (address, name, description) => {
    this.setState({
      showAdd: false
    });

    if (!address) {
      return;
    }

    this.updateDetails(address, name, description);
  }

  onCloseEdit = (address, name, description) => {
    this.setState({
      showEdit: false
    });

    if (!address) {
      return;
    }

    this.updateDetails(address, name, description);
  }

  onClose = () => {
    this.props.onClose();
  }

  wrapOnEdit (editing) {
    return () => {
      this.setState({
        editing,
        showEdit: true
      });
    };
  }
}
