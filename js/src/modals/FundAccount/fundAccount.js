import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import Modal from '../../ui/Modal';

const STAGE_NAMES = ['fund account'];

export default class FundAccount extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    visible: PropTypes.bool.isRequired,
    onClose: PropTypes.func
  }

  state = {
    stage: 0
  }

  render () {
    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ this.state.stage }
        steps={ STAGE_NAMES }
        visible={ this.props.visible }>
        <div>
          Placeholder until such time as we have the ShapeShift.io integration going (just time, a scarce commodity)
        </div>
      </Modal>
    );
  }

  renderDialogActions () {
    return (
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />
    );
  }

  onClose = () => {
    this.setState({
      stage: 0
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }
}
