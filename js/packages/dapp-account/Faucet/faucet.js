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
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { txLink } from '@parity/etherscan/links';
import { Button, ModalBox, Portal, ShortenedHash } from '@parity/ui';
import { CloseIcon, DialIcon, DoneIcon, ErrorIcon, SendIcon } from '@parity/ui/Icons';

import Store from './store';

@observer
export default class Faucet extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    onClose: PropTypes.func.isRequired
  }

  store = new Store(this.props.netVersion, this.props.address);

  render () {
    const { error, isBusy, isCompleted } = this.store;

    let icon = <DialIcon />;

    if (isCompleted) {
      icon = error
        ? <ErrorIcon />
        : <DoneIcon />;
    }

    return (
      <Portal
        buttons={ this.renderActions() }
        busy={ isBusy }
        isSmallModal
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='faucet.title'
            defaultMessage='Kovan ETH Faucet'
          />
        }
      >
        <ModalBox
          icon={ icon }
          summary={
            isCompleted
              ? this.renderSummaryDone()
              : this.renderSummaryRequest()
          }
        />
      </Portal>
    );
  }

  renderActions = () => {
    const { canTransact, isBusy, isCompleted } = this.store;

    return isCompleted || isBusy
      ? (
        <Button
          disabled={ isBusy }
          icon={ <DoneIcon /> }
          key='done'
          label={
            <FormattedMessage
              id='faucet.buttons.done'
              defaultMessage='close'
            />
          }
          onClick={ this.onClose }
        />
      )
      : [
        <Button
          icon={ <CloseIcon /> }
          key='close'
          label={
            <FormattedMessage
              id='faucet.buttons.close'
              defaultMessage='close'
            />
          }
          onClick={ this.onClose }
        />,
        <Button
          disabled={ !canTransact }
          icon={ <SendIcon /> }
          key='request'
          label={
            <FormattedMessage
              id='faucet.buttons.request'
              defaultMessage='request'
            />
          }
          onClick={ this.onExecute }
        />
      ];
  }

  renderSummaryDone () {
    const { error, responseText, responseTxHash } = this.store;

    return (
      <div>
        <FormattedMessage
          id='faucet.summary.done'
          defaultMessage='Your Kovan ETH has been requested from the faucet which responded with -'
        />
        {
          error
            ? (
              <p>{ error }</p>
            )
            : (
              <p>
                <span>{ responseText }&nbsp;</span>
                <a href={ txLink(responseTxHash, false, '42') } target='_blank'>
                  <ShortenedHash data={ responseTxHash } />
                </a>
              </p>
            )
        }
      </div>
    );
  }

  renderSummaryRequest () {
    return (
      <FormattedMessage
        id='faucet.summary.info'
        defaultMessage='To request a deposit of Kovan ETH to this address, you need to ensure that the address is sms-verified on the mainnet. Once executed the faucet will deposit Kovan ETH into the current account.'
      />
    );
  }

  onClose = () => {
    this.props.onClose();
  }

  onExecute = () => {
    return this.store.makeItRain();
  }
}
