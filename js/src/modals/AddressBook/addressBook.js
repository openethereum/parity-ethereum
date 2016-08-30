import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import Modal from '../../ui/Modal';

export default class AddressBook extends Component {
  static propTypes = {
    onClose: PropTypes.func
  };

  state = {
  };

  render () {
    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }>
        { this.renderPage() }
      </Modal>
    );
  }

  renderDialogActions () {
    return (
      <FlatButton
        icon={ <ContentClear /> }
        label='Close'
        primary
        onTouchTap={ this.onClose } />
    );
  }

  renderPage () {
    return <div>content</div>;
  }

  onClose = () => {
    this.props.onClose();
  }
}
