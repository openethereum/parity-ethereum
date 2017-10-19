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

import { STEP_COMPLETED, STEP_ERROR, STEP_INFO, STEP_UPDATING } from '@parity/shared/mobx/upgradeParity';
import Button from '@parity/ui/Button';
import Portal from '@parity/ui/Portal';
import { CancelIcon, DoneIcon, ErrorIcon, NextIcon, UpdateIcon, UpdateWaitIcon } from '@parity/ui/Icons';

import styles from './upgradeParity.css';

@observer
export default class UpgradeParity extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    upgradeStore: PropTypes.object.isRequired
  }

  render () {
    const { upgradeStore } = this.props;

    if (!upgradeStore.isVisible) {
      return null;
    }

    return (
      <Portal
        activeStep={ upgradeStore.step }
        busySteps={ [ 1 ] }
        buttons={ this.renderActions() }
        onClose={ this.onClose }
        open
        steps={ [
          <FormattedMessage
            id='upgradeParity.step.info'
            key='info'
            defaultMessage='upgrade available'
          />,
          <FormattedMessage
            key='updating'
            id='upgradeParity.step.updating'
            defaultMessage='upgrading parity'
          />,
          upgradeStore.error
            ? <FormattedMessage
              id='upgradeParity.step.error'
              key='error'
              defaultMessage='error'
              />
            : <FormattedMessage
              id='upgradeParity.step.completed'
              key='completed'
              defaultMessage='upgrade completed'
              />
        ] }
      >
        { this.renderStep() }
      </Portal>
    );
  }

  renderActions () {
    const { upgradeStore } = this.props;

    const closeButton =
      <Button
        icon={ <CancelIcon /> }
        key='close'
        label={
          <FormattedMessage
            id='upgradeParity.button.close'
            defaultMessage='close'
          />
        }
        onClick={ this.onClose }
      />;
    const doneButton =
      <Button
        icon={ <DoneIcon /> }
        key='done'
        label={
          <FormattedMessage
            id='upgradeParity.button.done'
            defaultMessage='done'
          />
        }
        onClick={ this.onDone }
      />;

    switch (upgradeStore.step) {
      case STEP_INFO:
        return [
          <Button
            icon={ <NextIcon /> }
            key='upgrade'
            label={
              <FormattedMessage
                id='upgradeParity.button.upgrade'
                defaultMessage='upgrade now'
              />
            }
            onClick={ this.onUpgrade }
          />,
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
    const { upgradeStore } = this.props;
    const currentversion = this.formatVersion(upgradeStore);
    const newversion = upgradeStore.upgrading
      ? this.formatVersion(upgradeStore.upgrading)
      : this.formatVersion(upgradeStore.available);

    switch (upgradeStore.step) {
      case STEP_INFO:
        return this.renderStepInfo(newversion, currentversion);

      case STEP_UPDATING:
        return this.renderStepBusy(newversion);

      case STEP_COMPLETED:
      case STEP_ERROR:
        return upgradeStore.error
          ? this.renderStepError(newversion)
          : this.renderStepCompleted(newversion);
    }
  }

  renderStepBusy (newversion) {
    return (
      <div className={ styles.step }>
        <UpdateWaitIcon className={ styles.icon } />
        <div className={ styles.text }>
          <FormattedMessage
            id='upgradeParity.busy'
            defaultMessage='Your upgrade to Parity {newversion} is currently in progress. Please wait until the process completes.'
            values={ {
              newversion: <div className={ styles.version }>{ newversion }</div>
            } }
          />
        </div>
      </div>
    );
  }

  renderStepCompleted (newversion) {
    return (
      <div className={ styles.step }>
        <DoneIcon className={ styles.icon } />
        <div className={ styles.text }>
          <FormattedMessage
            id='upgradeParity.completed'
            defaultMessage='Your upgrade to Parity {newversion} has been successfully completed. Click "done" to automatically reload the application.'
            values={ {
              newversion: <div className={ styles.version }>{ newversion }</div>
            } }
          />
        </div>
      </div>
    );
  }

  renderStepError (newversion) {
    const { upgradeStore } = this.props;

    return (
      <div className={ styles.step }>
        <ErrorIcon className={ styles.icon } />
        <div className={ styles.text }>
          <FormattedMessage
            id='upgradeParity.failed'
            defaultMessage='Your upgrade to Parity {newversion} has failed with an error.'
            values={ {
              newversion: <div className={ styles.version }>{ newversion }</div>
            } }
          />
          <div className={ styles.error }>
            { upgradeStore.error.message }
          </div>
        </div>
      </div>
    );
  }

  renderStepInfo (newversion, currentversion) {
    return (
      <div className={ styles.step }>
        <UpdateIcon className={ styles.icon } />
        <div className={ styles.text }>
          <div>
            <FormattedMessage
              id='upgradeParity.info.welcome'
              defaultMessage='Welcome to the Parity upgrade wizard, allowing you a completely seamless upgrade experience to the next version of Parity.'
            />
          </div>
          <div>
            <ul>
              <li>
                <FormattedMessage
                  id='upgradeParity.info.currentVersion'
                  defaultMessage='You are currently running {currentversion}'
                  values={ {
                    currentversion: <div className={ styles.version }>{ currentversion }</div>
                  } }
                />
              </li>
              <li>
                <FormattedMessage
                  id='upgradeParity.info.upgrade'
                  defaultMessage='An upgrade to version {newversion} is available'
                  values={ {
                    currentversion: <div className={ styles.version }>{ currentversion }</div>,
                    newversion: <div className={ styles.version }>{ newversion }</div>
                  } }
                />
              </li>
              <li>
                { this.renderConsensusInfo() }
              </li>
            </ul>
          </div>
          <div>
            <FormattedMessage
              id='upgradeParity.info.next'
              defaultMessage='Proceed with "upgrade now" to start your Parity upgrade.'
            />
          </div>
        </div>
      </div>
    );
  }

  renderConsensusInfo () {
    const { upgradeStore } = this.props;
    const { consensusCapability } = upgradeStore;

    if (consensusCapability) {
      if (consensusCapability === 'capable') {
        return (
          <FormattedMessage
            id='upgradeParity.consensus.capable'
            defaultMessage='Your current Parity version is capable of handling the network requirements.'
          />
        );
      } else if (consensusCapability.capableUntil) {
        return (
          <FormattedMessage
            id='upgradeParity.consensus.capableUntil'
            defaultMessage='Your current Parity version is capable of handling the network requirements until block {blockNumber}'
            values={ {
              blockNumber: consensusCapability.capableUntil
            } }
          />
        );
      } else if (consensusCapability.incapableSince) {
        return (
          <FormattedMessage
            id='upgradeParity.consensus.incapableSince'
            defaultMessage='Your current Parity version is incapable of handling the network requirements since block {blockNumber}'
            values={ {
              blockNumber: consensusCapability.incapableSince
            } }
          />
        );
      }
    }

    return (
      <FormattedMessage
        id='upgradeParity.consensus.unknown'
        defaultMessage='Your current Parity version is capable of handling the network requirements.'
      />
    );
  }

  formatVersion (struct) {
    if (!struct || !struct.version) {
      return (
        <FormattedMessage
          id='upgradeParity.version.unknown'
          defaultMessage='unknown'
        />
      );
    }

    const { track, version } = struct.version;

    return `${version.major}.${version.minor}.${version.patch}-${track}`;
  }

  onClose = () => {
    this.props.upgradeStore.closeModal();
  }

  onDone = () => {
    if (this.props.upgradeStore.error) {
      this.onClose();
    } else {
      window.location.reload();
    }
  }

  onUpgrade = () => {
    this.props.upgradeStore.upgradeNow();
  }
}
