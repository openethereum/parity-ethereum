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

import bytes from 'bytes';
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { observer } from 'mobx-react';

import { BlockNumber, BlockTimestamp, Container, ContainerTitle, Input, NetPeers } from '@parity/ui';

import MiningSettings from '../MiningSettings';
import StatusStore from './store';

import styles from './node.css';

@observer
export default class Node extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  statusStore = new StatusStore(this.context.api);

  render () {
    const { hashrate } = this.statusStore;
    const hashrateValue = bytes(hashrate.toNumber()) || 0;

    return (
      <Container>
        <div className={ styles.container }>
          <div className={ styles.row }>
            <div className={ styles.col3 }>
              <div className={ `${styles.col12} ${styles.padBottom}` }>
                <ContainerTitle
                  title={
                    <FormattedMessage
                      id='status.status.title.bestBlock'
                      defaultMessage='best block'
                    />
                  }
                />
                <div className={ styles.blockInfo }>
                  #<BlockNumber />
                </div>
                <div className={ styles.blockByline }>
                  <BlockTimestamp />
                </div>
              </div>
              <div className={ `${styles.col12} ${styles.padBottom}` }>
                <ContainerTitle
                  title={
                    <FormattedMessage
                      id='status.status.title.peers'
                      defaultMessage='peers'
                    />
                  }
                />
                <div className={ styles.blockInfo }>
                  <NetPeers />
                </div>
              </div>
              <div className={ `${styles.col12} ${styles.padBottom}` }>
                <ContainerTitle
                  title={
                    <FormattedMessage
                      id='status.status.title.hashRate'
                      defaultMessage='hash rate'
                    />
                  }
                />
                <div className={ styles.blockInfo }>
                  <FormattedMessage
                    id='status.status.hashrate'
                    defaultMessage='{hashrate} H/s'
                    values={ {
                      hashrate: hashrateValue
                    } }
                  />
                </div>
              </div>
            </div>
            <div className={ styles.col4_5 }>
              { this.renderMiningSettings() }
            </div>
            <div className={ styles.col4_5 }>
              { this.renderSettings() }
            </div>
          </div>
        </div>
      </Container>
    );
  }

  renderMiningSettings () {
    const { coinbase, defaultExtraData, extraData, gasFloorTarget, minGasPrice } = this.statusStore;

    return (
      <MiningSettings
        coinbase={ coinbase }
        defaultExtraData={ defaultExtraData }
        extraData={ extraData }
        gasFloorTarget={ gasFloorTarget }
        minGasPrice={ minGasPrice }
        onUpdateSetting={ this.statusStore.handleUpdateSetting }
      />
    );
  }

  renderNodeName () {
    const { nodeName } = this.statusStore;

    return (
      <span>
        { nodeName || (
          <FormattedMessage
            id='status.status.title.node'
            defaultMessage='Node'
          />)
        }
      </span>
    );
  }

  renderSettings () {
    const { enode, rpcSettings, netPort = '' } = this.statusStore;

    if (!rpcSettings) {
      return null;
    }

    const rpcPort = rpcSettings.port || '';

    return (
      <div>
        <ContainerTitle
          title={
            <FormattedMessage
              id='status.status.title.network'
              defaultMessage='network settings'
            />
          }
        />
        <Input
          allowCopy
          readOnly
          label={
            <FormattedMessage
              id='status.status.input.chain'
              defaultMessage='chain'
            />
          }
          value={ this.statusStore.netChain }
        />
        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label={
                <FormattedMessage
                  id='status.status.input.rpcEnabled'
                  defaultMessage='rpc enabled'
                />
              }
              value={
                rpcSettings.enabled
                  ? (
                    <FormattedMessage
                      id='status.status.input.yes'
                      defaultMessage='yes'
                    />
                  )
                  : (
                    <FormattedMessage
                      id='status.status.input.no'
                      defaultMessage='no'
                    />
                  )
              }
            />
          </div>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label={
                <FormattedMessage
                  id='status.status.input.port'
                  defaultMessage='network port'
                />
              }
              value={ netPort.toString() }
            />
          </div>
        </div>

        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label={
                <FormattedMessage
                  id='status.status.input.rpcInterface'
                  defaultMessage='rpc interface'
                />
              }
              value={ rpcSettings.interface }
            />
          </div>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label={
                <FormattedMessage
                  id='status.status.input.rpcPort'
                  defaultMessage='rpc port'
                />
              }
              value={ rpcPort.toString() }
            />
          </div>
        </div>

        <div className={ styles.row }>
          <div className={ styles.col12 }>
            <Input
              allowCopy
              readOnly
              label={
                <FormattedMessage
                  id='status.status.input.enode'
                  defaultMessage='enode'
                />
              }
              value={ enode }
            />
          </div>
        </div>
      </div>
    );
  }
}
