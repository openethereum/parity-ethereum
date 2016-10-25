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
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Button, IdentityIcon, Modal } from '../../ui';
import initShapeshift from '../../3rdparty/shapeshift';
import shapeshiftLogo from '../../../assets/images/shapeshift-logo.png';

import AwaitingDepositStep from './AwaitingDepositStep';
import AwaitingExchangeStep from './AwaitingExchangeStep';
import CompletedStep from './CompletedStep';
import ErrorStep from './ErrorStep';
import OptionsStep from './OptionsStep';

import styles from './shapeshift.css';

const shapeshift = initShapeshift();

const STAGE_NAMES = ['details', 'awaiting deposit', 'awaiting exchange', 'completed'];

export default class Shapeshift extends Component {
  static contextTypes = {
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    onClose: PropTypes.func
  }

  state = {
    stage: 0,
    coinSymbol: 'BTC',
    coinPair: 'btc_eth',
    coins: [],
    depositAddress: '',
    refundAddress: '',
    price: null,
    depositInfo: null,
    exchangeInfo: null,
    error: {},
    hasAccepted: false,
    shifting: false
  }

  componentDidMount () {
    this.retrieveCoins();
  }

  render () {
    const { error, stage } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ error.fatal ? null : STAGE_NAMES }
        title={ error.fatal ? 'exchange failed' : null }
        waiting={ [1, 2] }
        visible>
        { this.renderPage() }
      </Modal>
    );
  }

  renderDialogActions () {
    const { address } = this.props;
    const { coins, error, stage, hasAccepted, shifting } = this.state;

    const logo = (
      <a href='http://shapeshift.io' target='_blank' className={ styles.shapeshift }>
        <img src={ shapeshiftLogo } />
      </a>
    );
    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose } />
    );

    if (error.fatal) {
      return [
        logo,
        cancelBtn
      ];
    }

    switch (stage) {
      case 0:
        return [
          logo,
          cancelBtn,
          <Button
            disabled={ !coins.length || !hasAccepted || shifting }
            icon={ <IdentityIcon address={ address } button /> }
            label='Shift Funds'
            onClick={ this.onShift } />
        ];

      case 1:
      case 2:
        return [
          logo,
          cancelBtn
        ];

      case 3:
        return [
          logo,
          <Button
            icon={ <ActionDoneAll /> }
            label='Close'
            onClick={ this.onClose } />
        ];
    }
  }

  renderPage () {
    const { error, stage } = this.state;

    if (error.fatal) {
      return (
        <ErrorStep error={ error } />
      );
    }

    switch (stage) {
      case 0:
        return (
          <OptionsStep
            { ...this.state }
            onChangeSymbol={ this.onChangeSymbol }
            onChangeRefund={ this.onChangeRefund }
            onToggleAccept={ this.onToggleAccept } />
        );

      case 1:
        return (
          <AwaitingDepositStep { ...this.state } />
        );

      case 2:
        return (
          <AwaitingExchangeStep { ...this.state } />
        );

      case 3:
        return (
          <CompletedStep { ...this.state } />
        );
    }
  }

  setStage (stage) {
    this.setState({
      stage,
      error: {}
    });
  }

  setFatalError (message) {
    this.setState({
      stage: 0,
      error: {
        fatal: true,
        message
      }
    });
  }

  onClose = () => {
    this.setStage(0);
    this.props.onClose && this.props.onClose();
  }

  onShift = () => {
    const { address } = this.props;
    const { coinPair, refundAddress } = this.state;

    this.setState({
      stage: 1,
      shifting: true
    });

    shapeshift
      .shift(address, refundAddress, coinPair)
      .then((result) => {
        console.log('onShift', result);
        const depositAddress = result.deposit;

        shapeshift.subscribe(depositAddress, this.onExchangeInfo);
        this.setState({ depositAddress });
      })
      .catch((error) => {
        console.error('onShift', error);
        const message = `Failed to start exchange: ${error.message}`;

        this.newError(new Error(message));
        this.setFatalError(message);
      });
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

  onExchangeInfo = (error, result) => {
    if (error) {
      console.error('onExchangeInfo', error);

      if (error.fatal) {
        this.setFatalError(error.message);
      }

      this.newError(error);
      return;
    }

    console.log('onExchangeInfo', result.status, result);

    switch (result.status) {
      case 'received':
        this.setState({ depositInfo: result });
        this.setStage(2);
        return;

      case 'complete':
        this.setState({ exchangeInfo: result });
        this.setStage(3);
        return;
    }
  }

  getPrice (coinPair) {
    shapeshift
      .getMarketInfo(coinPair)
      .then((price) => {
        this.setState({ price });
      })
      .catch((error) => {
        console.error('getPrice', error);
      });
  }

  retrieveCoins () {
    const { coinPair } = this.state;

    shapeshift
      .getCoins()
      .then((_coins) => {
        const coins = Object.values(_coins).filter((coin) => coin.status === 'available');

        this.getPrice(coinPair);
        this.setState({ coins });
      })
      .catch((error) => {
        console.error('retrieveCoins', error);
        const message = `Failed to retrieve available coins from ShapeShift.io: ${error.message}`;

        this.newError(new Error(message));
        this.setFatalError(message);
      });
  }

  newError (error) {
    const { store } = this.context;

    store.dispatch({ type: 'newError', error });
  }
}
