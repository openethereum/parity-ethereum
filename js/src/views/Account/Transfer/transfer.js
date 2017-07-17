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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { observer } from 'mobx-react';
import { pick } from 'lodash';

import { nullableProptype } from '@parity/shared/util/proptypes';
import { Button, IdentityIcon, Portal, Warning } from '@parity/ui';
import { newError } from '@parity/ui/Errors/actions';
import { CancelIcon, NextIcon, PrevIcon } from '@parity/ui/Icons';

import Details from './Details';
import Extras from './Extras';

import TransferStore, { WALLET_WARNING_SPENT_TODAY_LIMIT } from './store';
import styles from './transfer.css';

const STEP_DETAILS = 0;
const STEP_EXTRA = 1;

@observer
class Transfer extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    newError: PropTypes.func.isRequired,
    gasLimit: PropTypes.object.isRequired,

    account: PropTypes.object,
    balance: PropTypes.object,
    onClose: PropTypes.func,
    senders: nullableProptype(PropTypes.object),
    sendersBalances: nullableProptype(PropTypes.object),
    tokens: PropTypes.object,
    wallet: PropTypes.object
  }

  store = new TransferStore(this.context.api, this.props);

  render () {
    const { stage, steps } = this.store;

    return (
      <Portal
        activeStep={ stage }
        buttons={ this.renderDialogActions() }
        onClose={ this.handleClose }
        open
        steps={ steps }
      >
        { this.renderExceptionWarning() }
        { this.renderWalletWarning() }
        { this.renderPage() }
      </Portal>
    );
  }

  renderExceptionWarning () {
    const { errorEstimated } = this.store.gasStore;

    if (!errorEstimated) {
      return null;
    }

    return (
      <Warning warning={ errorEstimated } />
    );
  }

  renderWalletWarning () {
    const { walletWarning } = this.store;

    if (!walletWarning) {
      return null;
    }

    if (walletWarning === WALLET_WARNING_SPENT_TODAY_LIMIT) {
      const warning = (
        <FormattedMessage
          id='transfer.warning.wallet_spent_limit'
          defaultMessage='This transaction value is above the remaining daily limit. It will need to be confirmed by other owners.'
        />
      );

      return (
        <Warning warning={ warning } />
      );
    }

    return null;
  }

  renderAccount () {
    const { account } = this.props;

    return (
      <div className={ styles.hdraccount }>
        <div className={ styles.hdrimage }>
          <IdentityIcon
            address={ account.address }
            center
            inline
          />
        </div>
        <div className={ styles.hdrdetails }>
          <div className={ styles.hdrname }>
            { account.name || 'Unnamed' }
          </div>
          <div className={ styles.hdraddress }>
            { account.address }
          </div>
        </div>
      </div>
    );
  }

  renderPage () {
    const { extras, stage } = this.store;

    if (stage === STEP_DETAILS) {
      return this.renderDetailsPage();
    } else if (stage === STEP_EXTRA && extras) {
      return this.renderExtrasPage();
    }
  }

  renderDetailsPage () {
    const { account, balance, senders } = this.props;
    const { recipient, recipientError, sender, senderError } = this.store;
    const { valueAll, extras, token, total, totalError, value, valueError } = this.store;

    return (
      <Details
        address={ account.address }
        all={ valueAll }
        balance={ balance }
        extras={ extras }
        onChange={ this.store.onUpdateDetails }
        recipient={ recipient }
        recipientError={ recipientError }
        sender={ sender }
        senderError={ senderError }
        senders={ senders }
        token={ token }
        total={ total }
        totalError={ totalError }
        value={ value }
        valueError={ valueError }
        wallet={ account.wallet && this.props.wallet }
      />
    );
  }

  renderExtrasPage () {
    if (!this.store.gasStore.histogram) {
      return null;
    }

    const { isEth, data, dataError, total, totalError } = this.store;

    return (
      <Extras
        data={ data }
        dataError={ dataError }
        gasStore={ this.store.gasStore }
        isEth={ isEth }
        onChange={ this.store.onUpdateDetails }
        total={ total }
        totalError={ totalError }
      />
    );
  }

  renderDialogActions () {
    const { account } = this.props;
    const { extras, sending, stage } = this.store;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='transfer.buttons.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.handleClose }
      />
    );
    const nextBtn = (
      <Button
        disabled={ !this.store.isValid }
        icon={ <NextIcon /> }
        key='next'
        label={
          <FormattedMessage
            id='transfer.buttons.next'
            defaultMessage='Next'
          />
        }
        onClick={ this.store.onNext }
      />
    );
    const prevBtn = (
      <Button
        icon={ <PrevIcon /> }
        key='back'
        label={
          <FormattedMessage
            id='transfer.buttons.back'
            defaultMessage='Back'
          />
        }
        onClick={ this.store.onPrev }
      />
    );
    const sendBtn = (
      <Button
        disabled={ !this.store.isValid || sending }
        icon={
          <IdentityIcon
            address={ account.address }
            button
          />
        }
        key='send'
        label={
          <FormattedMessage
            id='transfer.buttons.send'
            defaultMessage='Send'
          />
        }
        onClick={ this.store.onSend }
      />
    );

    switch (stage) {
      case 0:
        return extras
          ? [cancelBtn, nextBtn]
          : [cancelBtn, sendBtn];
      case 1:
        return [cancelBtn, prevBtn, sendBtn];
      default:
        return [cancelBtn];
    }
  }

  handleClose = () => {
    this.store.handleClose();
  }
}

function mapStateToProps (initState, initProps) {
  const { tokens } = initState;
  const { address } = initProps.account;

  const isWallet = initProps.account && initProps.account.wallet;
  const wallet = isWallet
    ? initState.wallet.wallets[address]
    : null;

  const senders = isWallet
    ? Object
      .values(initState.personal.accounts)
      .filter((account) => wallet.owners.includes(account.address))
      .reduce((accounts, account) => {
        accounts[account.address] = account;
        return accounts;
      }, {})
    : null;

  return (state) => {
    const { gasLimit } = state.nodeStatus;
    const { balances } = state;

    const balance = balances[address];
    const sendersBalances = senders ? pick(balances, Object.keys(senders)) : null;

    return { balance, gasLimit, senders, sendersBalances, tokens, wallet };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Transfer);
