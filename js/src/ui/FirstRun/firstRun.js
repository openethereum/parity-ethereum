import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ActionDone from 'material-ui/svg-icons/action/done';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import Overlay from '../Overlay';

import Completed from './Completed';
import CreateAccount from './CreateAccount';
import RecoverAccount from './RecoverAccount';
import Welcome from './Welcome';

const STAGE_NAMES = ['welcome', 'new account', 'recovery', 'completed'];

export default class FirstRun extends Component {
  static propTypes = {
    visible: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired
  }

  state = {
    stage: 0
  }

  render () {
    return (
      <Overlay
        actions={ this.renderDialogActions() }
        current={ this.state.stage }
        steps={ STAGE_NAMES }
        visible={ this.props.visible }>
        <Welcome
          visible={ this.state.stage === 0 } />
        <CreateAccount
          visible={ this.state.stage === 1 } />
        <RecoverAccount
          accountName='Newly Created Name'
          accountAddress='0xF6ABb80F11f269e4500A05721680E0a3AB075Ecf'
          accountPhrase='twenty never horse quick battery foot staple rabbit skate chair'
          visible={ this.state.stage === 2 } />
        <Completed
          visible={ this.state.stage === 3 } />
      </Overlay>
    );
  }

  renderDialogActions () {
    switch (this.state.stage) {
      case 0:
      case 2:
        return (
          <FlatButton
            icon={ <NavigationArrowForward /> }
            label='Next'
            primary
            onTouchTap={ this.onBtnNext } />
        );
      case 1:
        return (
          <FlatButton
            icon={ <ActionDone /> }
            label='Create'
            primary
            onTouchTap={ this.onBtnNext } />
        );
      case 3:
        return (
          <FlatButton
            icon={ <ActionDoneAll /> }
            label='Close'
            primary
            onTouchTap={ this.onBtnClose } />
      );
    }
  }

  onBtnClose = () => {
    this.setState({
      stage: 0
    }, this.props.onClose);
  }

  onBtnNext = () => {
    this.setState({
      stage: this.state.stage + 1
    });
  }
}
