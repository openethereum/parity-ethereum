// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button, Modal, TxHash, BusyStep } from '~/ui';
import { CancelIcon, DoneIcon, NextIcon } from '~/ui/Icons';

import WalletType from './WalletType';
import WalletDetails from './WalletDetails';
import WalletInfo from './WalletInfo';
import CreateWalletStore from './createWalletStore';

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
          title={
            <FormattedMessage
              id='createWallet.rejected.title'
              defaultMessage='rejected'
            />
          }
          actions={ this.renderDialogActions() }
        >
          <BusyStep
            title={
              <FormattedMessage
                id='createWallet.rejected.message'
                defaultMessage='The deployment has been rejected'
              />
            }
            state={
              <FormattedMessage
                id='createWallet.rejected.state'
                defaultMessage='The wallet will not be created. You can safely close this window.'
              />
            }
          />
        </Modal>
      );
    }

    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ steps.map((step) => step.title) }
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
            title={
              <FormattedMessage
                id='createWallet.deployment.message'
                defaultMessage='The deployment is currently in progress'
              />
            }
            state={ this.store.deployState }
          >
            {
              this.store.txhash
                ? <TxHash hash={ this.store.txhash } />
                : null
              }
          </BusyStep>
        );

      case 'INFO':
        return (
          <WalletInfo
            accounts={ accounts }
            account={ this.store.wallet.account }
            address={ this.store.wallet.address }
            daylimit={ this.store.wallet.daylimit }
            deployed={ this.store.deployed }
            name={ this.store.wallet.name }
            owners={ this.store.wallet.owners.slice() }
            required={ this.store.wallet.required }
          />
        );

      case 'DETAILS':
        return (
          <WalletDetails
            accounts={ accounts }
            errors={ this.store.errors }
            onChange={ this.store.onChange }
            wallet={ this.store.wallet }
            walletType={ this.store.walletType }
          />
        );

      default:
      case 'TYPE':
        return (
          <WalletType
            onChange={ this.store.onTypeChange }
            type={ this.store.walletType }
          />
        );
    }
  }

  renderDialogActions () {
    const { step, hasErrors, rejected, onCreate, onNext, onAdd } = this.store;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        label={
          <FormattedMessage
            id='createWallet.button.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.onClose }
      />
    );

    const closeBtn = (
      <Button
        icon={ <CancelIcon /> }
        label={
          <FormattedMessage
            id='createWallet.button.close'
            defaultMessage='Close'
          />
        }
        onClick={ this.onClose }
      />
    );

    const doneBtn = (
      <Button
        icon={ <DoneIcon /> }
        label={
          <FormattedMessage
            id='createWallet.button.done'
            defaultMessage='Done'
          />
        }
        onClick={ this.onClose }
      />
    );

    const sendingBtn = (
      <Button
        icon={ <DoneIcon /> }
        label={
          <FormattedMessage
            id='createWallet.button.sending'
            defaultMessage='Sending...'
          />
        }
        disabled
      />
    );

    const nextBtn = (
      <Button
        icon={ <NextIcon /> }
        label={
          <FormattedMessage
            id='createWallet.button.next'
            defaultMessage='Next'
          />
        }
        onClick={ onNext }
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

      case 'DETAILS':
        if (this.store.walletType === 'WATCH') {
          return [ cancelBtn, (
            <Button
              disabled={ hasErrors }
              icon={ <NextIcon /> }
              label={
                <FormattedMessage
                  id='createWallet.button.add'
                  defaultMessage='Add'
                />
              }
              onClick={ onAdd }
            />
          ) ];
        }

        return [ cancelBtn, (
          <Button
            disabled={ hasErrors }
            icon={ <NextIcon /> }
            label={
              <FormattedMessage
                id='createWallet.button.create'
                defaultMessage='Create'
              />
            }
            onClick={ onCreate }
          />
        ) ];

      default:
      case 'TYPE':
        return [ cancelBtn, nextBtn ];
    }
  }

  onClose = () => {
    this.props.onClose();
  }
}
