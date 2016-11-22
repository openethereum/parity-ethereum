// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';
import { observer } from 'mobx-react';

import ActionDone from 'material-ui/svg-icons/action/done';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import { Button, Modal, TxHash, BusyStep } from '../../ui';

import WalletDetails from './WalletDetails';
import WalletInfo from './WalletInfo';
import CreateWalletStore from './createWalletStore';
// import styles from './createWallet.css';

@observer
export default class CreateWallet extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired
  };

  store = new CreateWalletStore(this.context.api, this.props.accounts);

  render () {
    const { stage, steps, waiting, rejected } = this.store;

    if (rejected) {
      return (
        <Modal
          visible
          title='rejected'
          actions={ this.renderDialogActions() }
        >
          <BusyStep
            title='The deployment has been rejected'
            state='The wallet will not be created. You can safely close this window.'
          />
        </Modal>
      );
    }

    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ steps }
        waiting={ waiting }
      >
        { this.renderPage() }
      </Modal>
    );
  }

  renderPage () {
    const { step } = this.store;
    const { accounts } = this.props;

    switch (step) {
      case 'DEPLOYMENT':
        return (
          <BusyStep
            title='The deployment is currently in progress'
            state={ this.store.deployState }
          >
            { this.store.txhash ? (<TxHash hash={ this.store.txhash } />) : null }
          </BusyStep>
        );

      case 'INFO':
        return (
          <WalletInfo
            accounts={ accounts }

            account={ this.store.wallet.account }
            address={ this.store.wallet.address }
            owners={ this.store.wallet.owners.slice() }
            required={ this.store.wallet.required }
            daylimit={ this.store.wallet.daylimit }
          />
        );

      default:
      case 'DETAILS':
        return (
          <WalletDetails
            accounts={ accounts }
            wallet={ this.store.wallet }
            errors={ this.store.errors }
            onChange={ this.store.onChange }
          />
        );
    }
  }

  renderDialogActions () {
    const { step, hasErrors, rejected, onCreate } = this.store;

    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose }
      />
    );

    const closeBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Close'
        onClick={ this.onClose }
      />
    );

    const doneBtn = (
      <Button
        icon={ <ActionDone /> }
        label='Done'
        onClick={ this.onClose }
      />
    );

    const sendingBtn = (
      <Button
        icon={ <ActionDone /> }
        label='Sending...'
        disabled
      />
    );

    const createBtn = (
      <Button
        icon={ <NavigationArrowForward /> }
        label='Create'
        disabled={ hasErrors }
        onClick={ onCreate }
      />
    );

    if (rejected) {
      return [ closeBtn ];
    }

    switch (step) {
      case 'DEPLOYMENT':
        return [ closeBtn, sendingBtn ];

      case 'INFO':
        return [ doneBtn ];

      default:
      case 'DETAILS':
        return [ cancelBtn, createBtn ];

    }
  }

  onClose = () => {
    this.props.onClose();
  }
}
