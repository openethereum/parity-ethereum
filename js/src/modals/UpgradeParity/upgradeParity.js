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
import { FormattedMessage } from 'react-intl';

import { Button } from '~/ui';
import { CancelIcon, DoneIcon, NextIcon } from '~/ui/Icons';
import Modal, { Busy, Completed } from '~/ui/Modal';

import { STEP_COMPLETED, STEP_ERROR, STEP_INFO, STEP_UPDATING } from './store';
import styles from './upgradeParity.css';

@observer
export default class UpgradeParity extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { store } = this.props;

    if (!store.isVisible) {
      return null;
    }

    return (
      <Modal
        actions={ this.renderActions() }
        current={ store.step }
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
          store.step === STEP_ERROR
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
        visible
      >
        { this.renderStep() }
      </Modal>
    );
  }

  renderActions () {
    const { store } = this.props;

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
        onClick={ store.closeModal }
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
        onClick={ store.closeModal }
      />;

    switch (store.step) {
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
            onClick={ store.upgradeNow }
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
    const { store } = this.props;

    const currentversion = this.formatVersion(store);
    const newversion = store.upgrading
      ? this.formatVersion(store.upgrading)
      : this.formatVersion(store.available);

    switch (store.step) {
      case STEP_INFO:
        return (
          <div className={ styles.infoStep }>
            <div>
              <FormattedMessage
                id='upgradeParity.info.upgrade'
                defaultMessage='A new version of Parity, version {newversion} is available as an upgrade from your current version {currentversion}'
                values={ {
                  currentversion: <div className={ styles.version }>{ currentversion }</div>,
                  newversion: <div className={ styles.version }>{ newversion }</div>
                } }
              />
            </div>
            { this.renderConsensusInfo() }
          </div>
        );

      case STEP_UPDATING:
        return (
          <Busy
            title={
              <FormattedMessage
                id='upgradeParity.busy'
                defaultMessage='Your upgrade to Parity {newversion} is currently in progress'
                values={ {
                  newversion: <div className={ styles.version }>{ newversion }</div>
                } }
              />
            }
          />
        );

      case STEP_COMPLETED:
      case STEP_ERROR:
        if (store.error) {
          return (
            <Completed>
              <div>
                <FormattedMessage
                  id='upgradeParity.failed'
                  defaultMessage='Your upgrade to Parity {newversion} has failed with an error.'
                  values={ {
                    newversion: <div className={ styles.version }>{ newversion }</div>
                  } }
                />
              </div>
              <div className={ styles.error }>
                { store.error.message }
              </div>
            </Completed>
          );
        }

        return (
          <Completed>
            <FormattedMessage
              id='upgradeParity.completed'
              defaultMessage='Your upgrade to Parity {newversion} has been successfully completed.'
              values={ {
                newversion: <div className={ styles.version }>{ newversion }</div>
              } }
            />
          </Completed>
        );
    }
  }

  renderConsensusInfo () {
    const { store } = this.props;
    const { consensusCapability } = store;

    if (consensusCapability) {
      if (consensusCapability === 'capable') {
        return (
          <div>
            <FormattedMessage
              id='upgradeParity.consensus.capable'
              defaultMessage='Your current Parity version is capable of handling the network requirements.'
            />
          </div>
        );
      } else if (consensusCapability.capableUntil) {
        return (
          <div>
            <FormattedMessage
              id='upgradeParity.consensus.capableUntil'
              defaultMessage='Your current Parity version is capable of handling the network requirements until block {blockNumber}'
              values={ {
                blockNumber: consensusCapability.capableUntil
              } }
            />
          </div>
        );
      } else if (consensusCapability.incapableSince) {
        return (
          <div>
            <FormattedMessage
              id='upgradeParity.consensus.incapableSince'
              defaultMessage='Your current Parity version is incapable of handling the network requirements since block {blockNumber}'
              values={ {
                blockNumber: consensusCapability.incapableSince
              } }
            />
          </div>
        );
      }
    }

    return (
      <div>
        <FormattedMessage
          id='upgradeParity.consensus.unknown'
          defaultMessage='Your current Parity version is capable of handling the network requirements.'
        />
      </div>
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
}
