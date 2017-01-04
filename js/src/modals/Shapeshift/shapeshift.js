// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import shapeshiftLogo from '~/../assets/images/shapeshift-logo.png';
import { Button, IdentityIcon, Modal } from '~/ui';
import { CancelIcon, DoneIcon } from '~/ui/Icons';

import AwaitingDepositStep from './AwaitingDepositStep';
import AwaitingExchangeStep from './AwaitingExchangeStep';
import CompletedStep from './CompletedStep';
import ErrorStep from './ErrorStep';
import OptionsStep from './OptionsStep';
import Store, { STAGE_COMPLETED, STAGE_OPTIONS, STAGE_WAIT_DEPOSIT, STAGE_WAIT_EXCHANGE } from './store';

import styles from './shapeshift.css';

const STAGE_TITLES = [
  <FormattedMessage
    id='shapeshift.title.details'
    defaultMessage='details' />,
  <FormattedMessage
    id='shapeshift.title.deposit'
    defaultMessage='awaiting deposit' />,
  <FormattedMessage
    id='shapeshift.title.exchange'
    defaultMessage='awaiting exchange' />,
  <FormattedMessage
    id='shapeshift.title.completed'
    defaultMessage='completed' />
];
const ERROR_TITLE = (
  <FormattedMessage
    id='shapeshift.title.error'
    defaultMessage='exchange failed' />
);

export default class Shapeshift extends Component {
  static contextTypes = {
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    onClose: PropTypes.func
  }

  store = new Store(this.props.address);

  componentDidMount () {
    this.retrieveCoins();
  }

  componentWillUnmount () {
    this.store.unsubscribe();
  }

  render () {
    const { error, stage } = this.store;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={
          error && error.fatal
            ? null
            : STAGE_TITLES
        }
        title={
          error && error.fatal
            ? ERROR_TITLE
            : null
        }
        visible
        waiting={ [
          STAGE_WAIT_DEPOSIT,
          STAGE_WAIT_EXCHANGE
        ] }>
        { this.renderPage() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { address } = this.props;
    const { coins, error, hasAccepted, stage } = this.store;

    const logo = (
      <a href='http://shapeshift.io' target='_blank' className={ styles.shapeshift }>
        <img src={ shapeshiftLogo } />
      </a>
    );
    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        label={
          <FormattedMessage
            id='shapeshift.button.cancel'
            defaultMessage='Cancel' />
        }
        onClick={ this.onClose } />
    );

    if (error && error.fatal) {
      return [
        logo,
        cancelBtn
      ];
    }

    switch (stage) {
      case STAGE_OPTIONS:
        return [
          logo,
          cancelBtn,
          <Button
            disabled={ !coins.length || !hasAccepted }
            icon={ <IdentityIcon address={ address } button /> }
            label={
              <FormattedMessage
                id='shapeshift.button.shift'
                defaultMessage='Shift Funds' />
            }
            onClick={ this.onShift } />
        ];

      case STAGE_WAIT_DEPOSIT:
      case STAGE_WAIT_EXCHANGE:
        return [
          logo,
          cancelBtn
        ];

      case STAGE_COMPLETED:
        return [
          logo,
          <Button
            icon={ <DoneIcon /> }
            label={
              <FormattedMessage
                id='shapeshift.button.done'
                defaultMessage='Close' />
            }
            onClick={ this.onClose } />
        ];
    }
  }

  renderPage () {
    const { error, stage } = this.store;

    if (error && error.fatal) {
      return (
        <ErrorStep error={ error } />
      );
    }

    switch (stage) {
      case STAGE_OPTIONS:
        return (
          <OptionsStep store={ this.store } />
        );

      case STAGE_WAIT_DEPOSIT:
        return (
          <AwaitingDepositStep store={ this.store } />
        );

      case STAGE_WAIT_EXCHANGE:
        return (
          <AwaitingExchangeStep store={ this.store } />
        );

      case STAGE_COMPLETED:
        return (
          <CompletedStep store={ this.store } />
        );
    }
  }

  setFatalError (message) {
    this.setState({
      stage: STAGE_OPTIONS,
      error: {
        fatal: true,
        message
      }
    });
  }

  onClose = () => {
    this.store.setStage(STAGE_OPTIONS);
    this.props.onClose && this.props.onClose();
  }

  onShift = () => {
    this.store.shift();
  }

  onChangeSymbol = (event, coinSymbol) => {
    const coinPair = `${coinSymbol.toLowerCase()}_eth`;

    this.setState({
      coinPair,
      coinSymbol,
      price: null
    });
    this.getPrice(coinPair);
  }

  onChangeRefund = (event, refundAddress) => {
    this.setState({ refundAddress });
  }

  onToggleAccept = () => {
    const { hasAccepted } = this.state;

    this.setState({
      hasAccepted: !hasAccepted
    });
  }

  retrieveCoins = () => {
    return this.store
      .retrieveCoins()
      .catch((error) => {
        this.newError(error);
      });
  }

  newError (error) {
    const { store } = this.context;

    store.dispatch({ type: 'newError', error });
  }
}
