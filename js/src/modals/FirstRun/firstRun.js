import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ActionDone from 'material-ui/svg-icons/action/done';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import { newError } from '../../ui/Errors';
import { Modal } from '../../ui';

import { NewAccount, AccountDetails } from '../CreateAccount';

import Completed from './Completed';
import Welcome from './Welcome';

const STAGE_NAMES = ['welcome', 'new account', 'recovery', 'completed'];

class FirstRun extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    visible: PropTypes.bool,
    onClose: PropTypes.func.isRequired,
    onNewError: PropTypes.func
  }

  state = {
    stage: 0,
    name: '',
    address: '',
    password: '',
    phrase: '',
    canCreate: false
  }

  render () {
    const { visible } = this.props;
    const { stage } = this.state;

    if (!visible) {
      return null;
    }

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ STAGE_NAMES }
        visible>
        { this.renderStage() }
      </Modal>
    );
  }

  renderStage () {
    const { stage } = this.state;

    switch (stage) {
      case 0:
        return (
          <Welcome />
        );
      case 1:
        return (
          <NewAccount
            onChange={ this.onChangeDetails } />
        );
      case 2:
        return (
          <AccountDetails
            address={ this.state.address }
            name={ this.state.name }
            phrase={ this.state.phrase } />
        );
      case 3:
        return (
          <Completed />
        );
    }
  }

  renderDialogActions () {
    const { stage } = this.state;

    switch (stage) {
      case 0:
      case 2:
        return (
          <FlatButton
            icon={ <NavigationArrowForward /> }
            label='Next'
            primary
            onTouchTap={ this.onNext } />
        );

      case 1:
        return (
          <FlatButton
            icon={ <ActionDone /> }
            label='Create'
            disabled={ !this.state.canCreate }
            primary
            onTouchTap={ this.onCreate } />
        );

      case 3:
        return (
          <FlatButton
            icon={ <ActionDoneAll /> }
            label='Close'
            primary
            onTouchTap={ this.onClose } />
      );
    }
  }

  onClose = () => {
    const { onClose } = this.props;

    this.setState({
      stage: 0
    }, onClose);
  }

  onNext = () => {
    const { stage } = this.state;

    this.setState({
      stage: stage + 1
    });
  }

  onChangeDetails = (valid, { name, address, password, phrase }) => {
    this.setState({
      canCreate: valid,
      name: name,
      address: address,
      password: password,
      phrase: phrase
    });
  }

  onCreate = () => {
    const { api } = this.context;

    this.setState({
      canCreate: false
    });

    return api.personal
      .newAccountFromPhrase(this.state.phrase, this.state.password)
      .then((address) => api.personal.setAccountName(address, this.state.name))
      .then(() => {
        this.onNext();
      })
      .catch((error) => {
        console.error('onCreate', error);

        this.setState({
          canCreate: true
        });

        this.newError(error);
      });
  }

  newError = (error) => {
    this.props.onNewError(error);
  }
}

function mapStateToProps (state) {
  return {};
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onNewError: newError
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(FirstRun);
