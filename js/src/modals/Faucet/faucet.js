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
import { connect } from 'react-redux';
import { FormattedMessage } from 'react-intl';

import { Button, Form, InputAddress, ModalBox, Portal } from '~/ui';
import { CloseIcon, DialIcon, DoneIcon, SendIcon } from '~/ui/Icons';

import Store from './store';

const ERROR_ADDRESS = (
  <FormattedMessage
    id='faucet.error.address'
    defaultMessage='not a valid network address'
  />
);

@observer
class Faucet extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    onClose: PropTypes.func.isRequired
  }

  store = new Store(this.props.netVersion, this.props.address);

  render () {
    const { isBusy } = this.store;

    return (
      <Portal
        buttons={ this.renderActions() }
        busy={ isBusy }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='faucet.title'
            defaultMessage='Kovan ETH Faucet'
          />
        }
      >
        { this.renderBody() }
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

  renderBody () {
    const { isBusy, isCompleted, isDestination } = this.store;

    if (isBusy) {
      return this.renderBodyBusy();
    }

    if (isCompleted) {
      return this.renderBodyDone();
    }

    return isDestination
      ? this.renderBodyKovan()
      : this.renderBodyFoundation();
  }

  renderBodyBusy () {
    return (
      <ModalBox
        icon={ <SendIcon /> }
        summary={
          <FormattedMessage
            id='faucet.summary.busy'
            defaultMessage='Requesting Kovan ETH from the Faucet. Please be patient while the process completes.'
          />
        }
      />
    );
  }

  renderBodyDone () {
    return (
      <ModalBox
        icon={ <DoneIcon /> }
        summary={
          <FormattedMessage
            id='faucet.summary.done'
            defaultMessage='Your ETH has been requested from the faucet. It should reflect in your account shortly.'
          />
        }
      />
    );
  }

  renderBodyFoundation () {
    const { accounts } = this.props;
    const { addressReceive, addressReceiveValid } = this.store;

    return (
      <ModalBox
        icon={ <DialIcon /> }
        summary={
          <FormattedMessage
            id='faucet.summary.foundation'
            defaultMessage='Request Kovan ETH from the faucet by executing the transfer for am sms-verified Foumdation address. By selecting a Kovan address below and executing, the faucet will deposit ETH into the address on Kovan.'
          />
        }
      >
        <Form>
          <InputAddress
            accounts={ accounts }
            autoFocus
            error={
              addressReceiveValid
                ? null
                : ERROR_ADDRESS
            }
            hint={
              <FormattedMessage
                id='faucet.input.kovan.hint'
                defaultMessage='Destination address on the Kovan network'
              />
            }
            label={
              <FormattedMessage
                id='faucet.input.kovan.label'
                defaultMessage='Kovan address'
              />
            }
            onChange={ this.onChangeAddressReceive }
            value={ addressReceive || '' }
          />
        </Form>
      </ModalBox>
    );
  }

  renderBodyKovan () {
    const { accounts } = this.props;
    const { addressVerified, addressVerifiedValid } = this.store;

    return (
      <ModalBox
        icon={ <DialIcon /> }
        summary={
          <FormattedMessage
            id='faucet.summary.kovan'
            defaultMessage='To request a deposit of Kovan ETH to this address, you need to select a sms-verified Foundation mainnet address. Once the request is executed and the Foundation address verified, the faucet will deposit ETH into the current account.'
          />
        }
      >
        <Form>
          <InputAddress
            accounts={ accounts }
            autoFocus
            error={
              addressVerifiedValid
                ? null
                : ERROR_ADDRESS
            }
            hint={
              <FormattedMessage
                id='faucet.input.foundation.hint'
                defaultMessage='An sms-verified address on the Foundation network'
              />
            }
            label={
              <FormattedMessage
                id='faucet.input.foundation.label'
                defaultMessage='Mainnet sms-verified address'
              />
            }
            onChange={ this.onChangeAddressVerified }
            value={ addressVerified || '' }
          />
        </Form>
      </ModalBox>
    );
  }

  onChangeAddressReceive = (event, address) => {
    this.store.setAddressReceive(address);
  }

  onChangeAddressVerified = (event, address) => {
    this.store.setAddressVerified(address);
  }

  onClose = () => {
    this.props.onClose();
  }

  onExecute = () => {
    return this.store.makeItRain();
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return {
    accounts
  };
}

export default connect(
  mapStateToProps,
  null
)(Faucet);
