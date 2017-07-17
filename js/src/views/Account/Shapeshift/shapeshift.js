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

import { Button, IdentityIcon, Portal } from '@parity/ui';
import { CancelIcon, DoneIcon } from '@parity/ui/Icons';

import shapeshiftLogo from '@parity/shared/assets/images/shapeshift-logo.png';

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
    defaultMessage='details'
  />,
  <FormattedMessage
    id='shapeshift.title.deposit'
    defaultMessage='awaiting deposit'
  />,
  <FormattedMessage
    id='shapeshift.title.exchange'
    defaultMessage='awaiting exchange'
  />,
  <FormattedMessage
    id='shapeshift.title.completed'
    defaultMessage='completed'
  />
];
const ERROR_TITLE = (
  <FormattedMessage
    id='shapeshift.title.error'
    defaultMessage='exchange failed'
  />
);

@observer
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
    this.store.retrieveCoins();
  }

  componentWillUnmount () {
    this.store.unsubscribe();
  }

  render () {
    const { error, stage } = this.store;

    return (
      <Portal
        activeStep={ stage }
        busySteps={ [
          STAGE_WAIT_DEPOSIT,
          STAGE_WAIT_EXCHANGE
        ] }
        buttons={ this.renderDialogActions() }
        onClose={ this.onClose }
        open
        steps={
          error
            ? null
            : STAGE_TITLES
        }
        title={
          error
            ? ERROR_TITLE
            : null
        }
      >
        { this.renderPage() }
      </Portal>
    );
  }

  renderDialogActions () {
    const { address } = this.props;
    const { coins, error, hasAcceptedTerms, stage } = this.store;

    const logo = (
      <a
        className={ styles.shapeshift }
        href='http://shapeshift.io'
        key='logo'
        target='_blank'
      >
        <img src={ shapeshiftLogo } />
      </a>
    );
    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='shapeshift.button.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.onClose }
      />
    );

    if (error) {
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
            disabled={ !coins.length || !hasAcceptedTerms }
            icon={
              <IdentityIcon
                address={ address }
                button
              />
            }
            key='shift'
            label={
              <FormattedMessage
                id='shapeshift.button.shift'
                defaultMessage='Shift Funds'
              />
            }
            onClick={ this.onShift }
          />
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
            key='done'
            label={
              <FormattedMessage
                id='shapeshift.button.done'
                defaultMessage='Close'
              />
            }
            onClick={ this.onClose }
          />
        ];
    }
  }

  renderPage () {
    const { error, stage } = this.store;

    if (error) {
      return (
        <ErrorStep store={ this.store } />
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

  onClose = () => {
    this.store.setStage(STAGE_OPTIONS);
    this.props.onClose && this.props.onClose();
  }

  onShift = () => {
    return this.store.shift();
  }
}
