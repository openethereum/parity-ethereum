import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Modal } from '../../ui';

const STAGE_NAMES = ['fund account'];

export default class FundAccount extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    onClose: PropTypes.func
  }

  state = {
    stage: 0
  }

  render () {
    const { stage } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ STAGE_NAMES }
        visible>
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
