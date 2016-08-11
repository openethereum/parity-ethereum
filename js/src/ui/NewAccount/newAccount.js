import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ActionDone from 'material-ui/svg-icons/action/done';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import NavigationArrowBack from 'material-ui/svg-icons/navigation/arrow-back';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import Overlay from '../Overlay';

import AccountDetails from './AccountDetails';
import CreationType from './CreationType';
import CreateAccount from './CreateAccount';
import ImportWallet from './ImportWallet';

const STAGE_NAMES = ['creation type', 'create account', 'account information'];
const STAGE_IMPORT = ['creation type', 'import wallet', 'account information'];

export default class NewAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    onClose: PropTypes.func,
    onUpdate: PropTypes.func,
    visible: PropTypes.bool.isRequired
  }

  state = {
    address: null,
    name: null,
    password: null,
    phrase: null,
    json: null,
    canCreate: false,
    createType: null,
    stage: 0
  }

  render () {
    return (
      <Overlay
        actions={ this.renderDialogActions() }
        current={ this.state.stage }
        steps={ this.state.createType === 'fromNew' ? STAGE_NAMES : STAGE_IMPORT }
        visible={ this.props.visible }>
        { this.renderPage() }
      </Overlay>
    );
  }

  renderPage () {
    switch (this.state.stage) {
      case 0:
        return (
          <CreationType
            onChange={ this.onChangeType } />
        );

      case 1:
        if (this.state.createType === 'fromNew') {
          return (
            <CreateAccount
              onChange={ this.onChangeDetails } />
          );
        } else {
          return (
            <ImportWallet
              onChange={ this.onChangeWallet } />
          );
        }

      case 2:
        return (
          <AccountDetails
            address={ this.state.address }
            name={ this.state.name }
            phrase={ this.state.phrase } />
        );
    }
  }

  renderDialogActions () {
    switch (this.state.stage) {
      case 0:
        return (
          <FlatButton
            icon={ <NavigationArrowForward /> }
            label='Next'
            primary
            onTouchTap={ this.onNext } />
        );
      case 1:
        return [
          <FlatButton
            icon={ <NavigationArrowBack /> }
            label='Back'
            primary
            onTouchTap={ this.onPrev } />,
          <FlatButton
            icon={ <ActionDone /> }
            label='Create'
            disabled={ !this.state.canCreate }
            primary
            onTouchTap={ this.onCreate } />
        ];
      case 2:
        return (
          <FlatButton
            icon={ <ActionDoneAll /> }
            label='Close'
            primary
            onTouchTap={ this.onClose } />
        );
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

  onCreate = () => {
    const api = this.context.api;

    if (this.state.createType === 'fromNew') {
      return api.personal
        .newAccountFromPhrase(this.state.phrase, this.state.password)
        .then((address) => api.personal.setAccountName(address, this.state.name))
        .then(() => {
          this.onNext();
          this.props.onUpdate && this.props.onUpdate();
        });
    }

    return api.personal
      .newAccountFromWallet(this.state.json, this.state.password)
      .then((address) => api.personal.setAccountName(address, this.state.name))
      .then(() => {
        this.onNext();
        this.props.onUpdate && this.props.onUpdate();
      });
  }

  onClose = () => {
    this.setState({
      stage: 0,
      canCreate: false
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }

  onChangeType = (value) => {
    this.setState({
      createType: value
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

  onChangeWallet = (valid, { name, password, json }) => {
    this.setState({
      canCreate: valid,
      name: name,
      password: password,
      json: json
    });
  }
}
