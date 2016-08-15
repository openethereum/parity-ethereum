import React, { Component, PropTypes } from 'react';
import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';
import ContentSend from 'material-ui/svg-icons/content/send';
import NavigationArrowBack from 'material-ui/svg-icons/navigation/arrow-back';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import Modal from '../../Modal';

import Details from './Details';
import Verify from './Verify';

const STAGE_NAMES = ['transfer', 'verify transaction', 'transaction receipt'];

export default class Transfer extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    visible: PropTypes.bool.isRequired,
    onClose: PropTypes.func
  }

  state = {
    stage: 0,
    recipient: null,
    isValid: false
  }

  render () {
    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ this.state.stage }
        steps={ STAGE_NAMES }
        visible={ this.props.visible }>
        { this.renderPage() }
      </Modal>
    );
  }

  renderPage () {
    switch (this.state.stage) {
      case 0:
        return (
          <Details
            address={ this.props.address }
            onChange={ this.onChangeDetails } />
        );
      case 1:
        return (
          <Verify
            address={ this.props.address }
            recipient={ this.state.recipient }
            onChange={ this.onChangePassword } />
        );
    }
  }

  renderDialogActions () {
    switch (this.state.stage) {
      case 0:
        return [
          <FlatButton
            icon={ <ContentClear /> }
            label='Cancel'
            primary
            onTouchTap={ this.onClose } />,
          <FlatButton
            disabled={ !this.state.isValid }
            icon={ <NavigationArrowForward /> }
            label='Next'
            primary
            onTouchTap={ this.onNext } />
        ];
      case 1:
        return [
          <FlatButton
            icon={ <ContentClear /> }
            label='Cancel'
            primary
            onTouchTap={ this.onClose } />,
          <FlatButton
            icon={ <NavigationArrowBack /> }
            label='Back'
            primary
            onTouchTap={ this.onPrev } />,
          <FlatButton
            disabled={ !this.state.isValid }
            icon={ <ContentSend /> }
            label='Send'
            primary
            onTouchTap={ this.onNext } />
        ];
    }
  }

  onNext = () => {
    this.setState({
      stage: this.state.stage + 1
    });
  }

  onPrev = () => {
    this.setState({
      stage: this.state.stage - 1
    });
  }

  onChangeDetails = (valid, { recipient }) => {
    this.setState({
      recipient: recipient,
      isValid: valid
    });
  }

  onChangePassword = (valid, { password }) => {
    this.setState({
      password: password,
      isValid: valid
    });
  }

  onClose = () => {
    this.setState({
      stage: 0
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }
}
