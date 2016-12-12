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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button, Modal } from '~/ui';
import { CancelIcon, DoneIcon, NextIcon, SnoozeIcon } from '~/ui/Icons';

import Info from './Info';
import Updating from './Updating';

import ModalStore, { STEP_COMPLETED, STEP_ERROR, STEP_INFO, STEP_UPDATING } from './modalStore';
import UpgradeStore from './upgradeStore';

@observer
export default class UpgradeParity extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  store = new ModalStore(new UpgradeStore(this.context.api));

  render () {
    if (!this.store.upgrade.available || !this.store.showUpgrade) {
      return null;
    }

    return (
      <Modal
        actions={ this.renderActions() }
        visible>
        { this.renderStep() }
      </Modal>
    );
  }

  renderActions () {
    const closeButton =
      <Button
        icon={ <CancelIcon /> }
        label={
          <FormattedMessage
            id='upgradeParity.button.close'
            defaultMessage='close' />
        }
        onClick={ this.store.closeModal } />;
    const doneButton =
      <Button
        icon={ <DoneIcon /> }
        label={
          <FormattedMessage
            id='upgradeParity.button.done'
            defaultMessage='done' />
        }
        onClick={ this.store.closeModal } />;

    switch (this.store.step) {
      case STEP_INFO:
        return [
          <Button
            icon={ <SnoozeIcon /> }
            label={
              <FormattedMessage
                id='upgradeParity.button.snooze'
                defaultMessage='ask me tomorrow' />
            }
            onClick={ this.store.snoozeTillTomorrow } />,
          <Button
            icon={ <NextIcon /> }
            label={
              <FormattedMessage
                id='upgradeParity.button.upgrade'
                defaultMessage='upgrade now' />
            }
            onClick={ this.store.upgradeNow } />,
          closeButton
        ];

      case STEP_UPDATING:
        return [
          closeButton
        ];

      case STEP_COMPLETED:
      case STEP_ERROR:
        return [
          doneButton
        ];
    }
  }

  renderStep () {
    switch (this.store.step) {
      case STEP_INFO:
        return <Info store={ this.store } />;

      case STEP_UPDATING:
        return <Updating store={ this.store } />;
    }
  }
}
