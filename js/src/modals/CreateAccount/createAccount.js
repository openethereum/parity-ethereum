import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ActionDone from 'material-ui/svg-icons/action/done';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowBack from 'material-ui/svg-icons/navigation/arrow-back';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import Modal from '../../ui/Modal';

import AccountDetails from './AccountDetails';
import AccountDetailsGeth from './AccountDetailsGeth';
import CreationType from './CreationType';
import NewAccount from './NewAccount';
import NewGeth from './NewGeth';
import NewImport from './NewImport';

const TITLES = {
  type: 'creation type',
  create: 'create account',
  info: 'account information',
  import: 'import wallet'
};
const STAGE_NAMES = [TITLES.type, TITLES.create, TITLES.info];
const STAGE_IMPORT = [TITLES.type, TITLES.import, TITLES.info];

export default class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    errorHandler: PropTypes.func.isRequired
  }

  static propTypes = {
    onClose: PropTypes.func,
    onUpdate: PropTypes.func
  }

  state = {
    address: null,
    name: null,
    passwordHint: null,
    password: null,
    phrase: null,
    json: null,
    canCreate: false,
    createType: null,
    gethAddresses: [],
    stage: 0
  }

  render () {
    const steps = this.state.createType === 'fromNew' ? STAGE_NAMES : STAGE_IMPORT;

    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        current={ this.state.stage }
        steps={ steps }>
        { this.renderPage() }
      </Modal>
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
            <NewAccount
              onChange={ this.onChangeDetails } />
          );
        } else if (this.state.createType === 'fromGeth') {
          return (
            <NewGeth
              onChange={ this.onChangeGeth } />
          );
        } else {
          return (
            <NewImport
              onChange={ this.onChangeWallet } />
          );
        }

      case 2:
        if (this.state.createType === 'fromGeth') {
          return (
            <AccountDetailsGeth
              addresses={ this.state.gethAddresses } />
          );
        }

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
        return [
          <FlatButton
            icon={ <ContentClear /> }
            label='Cancel'
            primary
            onTouchTap={ this.onClose } />,
          <FlatButton
            icon={ <NavigationArrowForward /> }
            label='Next'
            primary
            onTouchTap={ this.onNext } />
        ];
      case 1:
        const createLabel = this.state.createType === 'fromNew'
          ? 'Create'
          : 'Import';

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
            icon={ <ActionDone /> }
            label={ createLabel }
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

    this.setState({
      canCreate: false
    });

    if (this.state.createType === 'fromNew') {
      return api.personal
        .newAccountFromPhrase(this.state.phrase, this.state.password)
        .then((address) => {
          return api.personal
            .setAccountName(address, this.state.name)
            .then(() => api.personal.setAccountMeta(address, { passwordHint: this.state.passwordHint }));
        })
        .then(() => {
          this.onNext();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          this.setState({
            canCreate: true
          });

          this.context.errorHandler(error);
        });
    } else if (this.state.createType === 'fromGeth') {
      return api.personal
        .importGethAccounts(this.state.gethAddresses)
        .then((result) => {
          console.log('result', result);
          return Promise.all(this.state.gethAddresses.map((address) => {
            return api.personal.setAccountName(address, 'Geth Import');
          }));
        })
        .then(() => {
          this.onNext();
          this.props.onUpdate && this.props.onUpdate();
        })
        .catch((error) => {
          this.setState({
            canCreate: true
          });

          this.context.errorHandler(error);
        });
    }

    return api.personal
      .newAccountFromWallet(this.state.json, this.state.password)
      .then((address) => {
        this.setState({
          address: address
        });

        return api.personal
          .setAccountName(address, this.state.name)
          .then(() => api.personal.setAccountMeta(address, { passwordHint: this.state.passwordHint }));
      })
      .then(() => {
        this.onNext();
        this.props.onUpdate && this.props.onUpdate();
      })
      .catch((error) => {
        this.setState({
          canCreate: true
        });

        this.context.errorHandler(error);
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

  onChangeDetails = (valid, { name, passwordHint, address, password, phrase }) => {
    this.setState({
      canCreate: valid,
      name,
      passwordHint,
      address,
      password,
      phrase
    });
  }

  onChangeGeth = (valid, gethAddresses) => {
    this.setState({
      canCreate: valid,
      gethAddresses
    });
  }

  onChangeWallet = (valid, { name, passwordHint, password, json }) => {
    this.setState({
      canCreate: valid,
      name,
      passwordHint,
      password,
      json
    });
  }
}
