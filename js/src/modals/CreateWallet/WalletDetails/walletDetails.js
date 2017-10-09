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
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { Form, TypedInput, Input, AddressSelect, InputAddress } from '~/ui';

import styles from '../createWallet.css';

class WalletDetails extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    wallet: PropTypes.object.isRequired,
    errors: PropTypes.object.isRequired,
    onChange: PropTypes.func.isRequired,
    walletType: PropTypes.string.isRequired,

    knownAddresses: PropTypes.array
  };

  render () {
    const { walletType } = this.props;

    if (walletType === 'WATCH') {
      return this.renderWatchDetails();
    }

    return this.renderMultisigDetails();
  }

  renderWatchDetails () {
    const { wallet, errors } = this.props;

    return (
      <Form>
        <InputAddress
          autoFocus
          hint={
            <FormattedMessage
              id='createWallet.details.address.hint'
              defaultMessage='the wallet contract address'
            />
          }
          label={
            <FormattedMessage
              id='createWallet.details.address.label'
              defaultMessage='wallet address'
            />
          }
          value={ wallet.address }
          error={ errors.address }
          onChange={ this.onAddressChange }
        />

        <Input
          error={ errors.name }
          hint={
            <FormattedMessage
              id='createWallet.details.name.hint'
              defaultMessage='the local name for this wallet'
            />
          }
          label={
            <FormattedMessage
              id='createWallet.details.name.label'
              defaultMessage='wallet name'
            />
          }
          value={ wallet.name }
          onChange={ this.onNameChange }
        />

        <Input
          hint={
            <FormattedMessage
              id='createWallet.details.description.hint'
              defaultMessage='the local description for this wallet'
            />
          }
          label={
            <FormattedMessage
              id='createWallet.details.description.label'
              defaultMessage='wallet description (optional)'
            />
          }
          value={ wallet.description }
          onChange={ this.onDescriptionChange }
        />
      </Form>
    );
  }

  renderMultisigDetails () {
    const { accounts, knownAddresses, wallet, errors } = this.props;
    const allowedOwners = knownAddresses
      // Exclude sender and already owners of the wallet
      .filter((address) => !wallet.owners.includes(address) && address !== wallet.account);

    return (
      <Form>
        <Input
          autoFocus
          error={ errors.name }
          hint={
            <FormattedMessage
              id='createWallet.details.nameMulti.hint'
              defaultMessage='the local name for this wallet'
            />
          }
          label={
            <FormattedMessage
              id='createWallet.details.nameMulti.label'
              defaultMessage='wallet name'
            />
          }
          value={ wallet.name }
          onChange={ this.onNameChange }
        />

        <Input
          hint={
            <FormattedMessage
              id='createWallet.details.descriptionMulti.hint'
              defaultMessage='the local description for this wallet'
            />
          }
          label={
            <FormattedMessage
              id='createWallet.details.descriptionMulti.label'
              defaultMessage='wallet description (optional)'
            />
          }
          value={ wallet.description }
          onChange={ this.onDescriptionChange }
        />

        <AddressSelect
          accounts={ accounts }
          error={ errors.account }
          hint={
            <FormattedMessage
              id='createWallet.details.ownerMulti.hint'
              defaultMessage='the owner account for this contract'
            />
          }
          label={
            <FormattedMessage
              id='createWallet.details.ownerMulti.label'
              defaultMessage='from account (contract owner)'
            />
          }
          value={ wallet.account }
          onChange={ this.onAccoutChange }
        />

        <TypedInput
          allowedValues={ allowedOwners }
          label={
            <FormattedMessage
              id='createWallet.details.ownersMulti.label'
              defaultMessage='other wallet owners'
            />
          }
          onChange={ this.onOwnersChange }
          param='address[]'
          value={ wallet.owners.slice() }
        />

        <div className={ styles.splitInput }>
          <TypedInput
            error={ errors.required }
            hint={
              <FormattedMessage
                id='createWallet.details.ownersMultiReq.hint'
                defaultMessage='number of required owners to accept a transaction'
              />
            }
            label={
              <FormattedMessage
                id='createWallet.details.ownersMultiReq.label'
                defaultMessage='required owners'
              />
            }
            value={ wallet.required }
            onChange={ this.onRequiredChange }
            param='uint'
            min={ 1 }
            max={ wallet.owners.length + 1 }
          />

          <TypedInput
            error={ errors.daylimit }
            hint={
              <FormattedMessage
                id='createWallet.details.dayLimitMulti.hint'
                defaultMessage='amount of ETH spendable without confirmations'
              />
            }
            isEth
            label={
              <FormattedMessage
                id='createWallet.details.dayLimitMulti.label'
                defaultMessage='wallet day limit'
              />
            }
            onChange={ this.onDaylimitChange }
            param='uint'
            value={ wallet.daylimit }
          />
        </div>
      </Form>
    );
  }

  onAddressChange = (_, address) => {
    this.props.onChange({ address });
  }

  onAccoutChange = (_, account) => {
    this.props.onChange({ account });
  }

  onNameChange = (_, name) => {
    this.props.onChange({ name });
  }

  onDescriptionChange = (_, description) => {
    this.props.onChange({ description });
  }

  onOwnersChange = (owners) => {
    this.props.onChange({ owners });
  }

  onRequiredChange = (required) => {
    this.props.onChange({ required });
  }

  onDaylimitChange = (daylimit) => {
    this.props.onChange({ daylimit });
  }
}

function mapStateToProps (initState) {
  const { accounts, contacts, contracts } = initState.personal;
  const knownAddresses = [].concat(
    Object.keys(accounts),
    Object.keys(contacts),
    Object.keys(contracts)
  );

  return () => ({
    knownAddresses
  });
}

export default connect(
  mapStateToProps,
  null
)(WalletDetails);
