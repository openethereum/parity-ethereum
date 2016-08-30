import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';

import Modal from '../../ui/Modal';

import AddEntry from './AddEntry';
import EditEntry from './EditEntry';

export default class AddressBook extends Component {
  static propTypes = {
    onClose: PropTypes.func
  };

  state = {
    showAdd: false,
    showEdit: false
  };

  render () {
    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }>
        { this.renderModals() }
        { this.renderPage() }
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

  renderPage () {
    return <div>content</div>;
  }

  renderModals () {
    const { showAdd, showEdit } = this.state;

    if (showAdd) {
      return (
        <AddEntry
          onClose={ this.onCloseAdd } />
      );
    } else if (showEdit) {
      return (
        <EditEntry
          onClose={ this.onCloseEdit } />
      );
    }

    return null;
  }

  onAdd = () => {
    console.log('onAdd');
    this.setState({
      showAdd: true
    });
  }

  onCloseAdd = () => {
    this.setState({
      showAdd: false
    });
  }

  onCloseEdit = () => {
    this.setState({
      showAdd: false
    });
  }

  onClose = () => {
    this.props.onClose();
  }
}
