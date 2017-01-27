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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { observer } from 'mobx-react';
import { pick } from 'lodash';

import ActionDone from 'material-ui/svg-icons/action/done';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import { Button, Modal, TxHash, BusyStep, Form, TypedInput, InputAddress, AddressSelect } from '~/ui';
import { fromWei } from '~/api/util/wei';

import WalletSettingsStore from './walletSettingsStore.js';
import styles from './walletSettings.css';

@observer
class WalletSettings extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accountsInfo: PropTypes.object.isRequired,
    wallet: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired,
    senders: PropTypes.object.isRequired
  };

  store = new WalletSettingsStore(this.context.api, this.props.wallet);

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
            title='The modifications have been rejected'
            state='The wallet settings will not be modified. You can safely close this window.'
          />
        </Modal>
      );
    }

    return (
      <Modal
        visible
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ steps.map((s) => s.title) }
        waiting={ waiting }
      >
        { this.renderPage() }
      </Modal>
    );
  }

  renderPage () {
    const { step } = this.store;

    switch (step) {
      case 'SENDING':
        return (
          <BusyStep
            title='The modifications are currently being sent'
            state={ this.store.deployState }
          >
            {
              this.store.requests.map((req) => {
                const key = req.id;

                if (req.txhash) {
                  return (<TxHash key={ key } hash={ req.txhash } />);
                }

                if (req.rejected) {
                  return (<p key={ key }>The transaction #{parseInt(key, 16)} has been rejected</p>);
                }
              })
            }
          </BusyStep>
        );

      case 'CONFIRMATION':
        const { changes } = this.store;

        return (
          <div>
            { this.renderChanges(changes) }
          </div>
        );

      default:
      case 'EDIT':
        const { wallet, errors } = this.store;
        const { accountsInfo, senders } = this.props;

        return (
          <Form>
            <p>
              In order to edit this contract's settings, at
              least { this.store.initialWallet.require.toNumber() } owners have to
              send the very same modifications.
              Otherwise, no modification will be taken into account...
            </p>

            <AddressSelect
              label='from account (wallet owner)'
              hint='send modifications as this owner'
              value={ wallet.sender }
              error={ errors.sender }
              onChange={ this.store.onSenderChange }
              accounts={ senders }
            />

            <TypedInput
              label='other wallet owners'
              value={ wallet.owners.slice() }
              onChange={ this.store.onOwnersChange }
              accounts={ accountsInfo }
              param='address[]'
            />

            <div className={ styles.splitInput }>
              <TypedInput
                label='required owners'
                hint='number of required owners to accept a transaction'
                error={ errors.require }
                min={ 1 }
                onChange={ this.store.onRequireChange }
                max={ wallet.owners.length }
                param='uint'
                value={ wallet.require }
              />

              <TypedInput
                label='wallet day limit'
                hint='amount of ETH spendable without confirmations'
                value={ wallet.dailylimit }
                error={ errors.dailylimit }
                onChange={ this.store.onDailylimitChange }
                param='uint'
                isEth
              />
            </div>
          </Form>
        );
    }
  }

  renderChanges (changes) {
    if (changes.length === 0) {
      return (
        <p>No modifications have been made to the Wallet settings.</p>
      );
    }

    const modifications = changes.map((change, index) => (
      <div key={ `${change.type}_${index}` }>
        { this.renderChange(change) }
      </div>
    ));

    return (
      <div>
        <p>You are about to make the following modifications</p>
        { modifications }
      </div>
    );
  }

  renderChange (change) {
    const { accountsInfo } = this.props;

    switch (change.type) {
      case 'dailylimit':
        return (
          <div className={ styles.change }>
            <div className={ styles.label }>Change Daily Limit</div>
            <div>
              <span> from </span>
              <code> { fromWei(change.initial).toFormat() }</code>
              <span className={ styles.eth } />
              <span> to </span>
              <code> { fromWei(change.value).toFormat() }</code>
              <span className={ styles.eth } />
            </div>
          </div>
        );

      case 'require':
        return (
          <div className={ styles.change }>
            <div className={ styles.label }>Change Required Owners</div>
            <div>
              <span> from </span>
              <code> { change.initial.toNumber() }</code>
              <span> to </span>
              <code> { change.value.toNumber() }</code>
            </div>
          </div>
        );

      case 'add_owner':
        return (
          <div className={ [ styles.change, styles.add ].join(' ') }>
            <div className={ styles.label }>Add Owner</div>
            <div>
              <InputAddress
                disabled
                value={ change.value }
                accounts={ accountsInfo }
              />
            </div>
          </div>
        );

      case 'remove_owner':
        return (
          <div className={ [ styles.change, styles.remove ].join(' ') }>
            <div className={ styles.label }>Remove Owner</div>
            <div>
              <InputAddress
                disabled
                value={ change.value }
                accounts={ accountsInfo }
              />
            </div>
          </div>
        );
    }
  }

  renderDialogActions () {
    const { onClose } = this.props;
    const { step, hasErrors, rejected, onNext, send, done } = this.store;

    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ onClose }
      />
    );

    const closeBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Close'
        onClick={ onClose }
      />
    );

    const sendingBtn = (
      <Button
        icon={ <ActionDone /> }
        label='Sending...'
        disabled
      />
    );

    const nextBtn = (
      <Button
        icon={ <NavigationArrowForward /> }
        label='Next'
        onClick={ onNext }
        disabled={ hasErrors }
      />
    );

    const sendBtn = (
      <Button
        icon={ <NavigationArrowForward /> }
        label='Send'
        onClick={ send }
        disabled={ hasErrors }
      />
    );

    if (rejected) {
      return [ closeBtn ];
    }

    switch (step) {
      case 'SENDING':
        return done ? [ closeBtn ] : [ closeBtn, sendingBtn ];

      case 'CONFIRMATION':
        const { changes } = this.store;

        if (changes.length === 0) {
          return [ closeBtn ];
        }

        return [ cancelBtn, sendBtn ];

      default:
      case 'TYPE':
        return [ cancelBtn, nextBtn ];
    }
  }
}

function mapStateToProps (initState, initProps) {
  const { accountsInfo, accounts } = initState.personal;
  const { owners } = initProps.wallet;

  const senders = pick(accounts, owners);

  return () => {
    return { accountsInfo, senders };
  };
}

export default connect(
  mapStateToProps,
  null
)(WalletSettings);
