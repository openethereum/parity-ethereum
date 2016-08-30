import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';
import ContentClear from 'material-ui/svg-icons/content/clear';

import Modal from '../../../ui/Modal';

export default class AddEntry extends Component {
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
    return ([
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />,
      <FlatButton
        icon={ <ContentAdd /> }
        label='Save Entry'
        primary
        onTouchTap={ this.onAdd } />
    ]);
  }

  renderPage () {
    return <div>content</div>;
  }
}
