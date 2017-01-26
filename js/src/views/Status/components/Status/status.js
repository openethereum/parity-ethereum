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
import moment from 'moment';
import React, { Component, PropTypes } from 'react';

import { Container, ContainerTitle, Input } from '~/ui';

import MiningSettings from '../MiningSettings';

import styles from './status.css';

export default class Status extends Component {
  static propTypes = {
    nodeStatus: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired
  }

  render () {
    const { nodeStatus } = this.props;
    const { netPeers } = nodeStatus;

    if (!netPeers || !nodeStatus.blockNumber) {
      return null;
    }

    const hashrate = bytes(nodeStatus.hashrate.toNumber()) || 0;
    const peers = `${netPeers.active}/${netPeers.connected}/${netPeers.max}`;

    return (
      <Container>
        <div className={ styles.container }>
          <div className={ styles.row }>
            <div className={ styles.col3 }>
              <div className={ `${styles.col12} ${styles.padBottom}` }>
                <ContainerTitle title='best block' />
                <div { ...this._test('best-block') } className={ styles.blockInfo }>
                  #{ nodeStatus.blockNumber.toFormat() }
                </div>
                <div className={ styles.blockByline }>
                  { moment(nodeStatus.blockTimestamp).calendar() }
                </div>
              </div>
              <div className={ `${styles.col12} ${styles.padBottom}` }>
                <ContainerTitle title='peers' />
                <div { ...this._test('peers') } className={ styles.blockInfo }>
                  { peers }
                </div>
              </div>
              <div className={ `${styles.col12} ${styles.padBottom}` }>
                <ContainerTitle title='hash rate' />
                <div { ...this._test('hashrate') } className={ styles.blockInfo }>
                  { `${hashrate} H/s` }
                </div>
              </div>
            </div>
            <div className={ styles.col4_5 }>
              <MiningSettings
                { ...this._test('mining') }
                nodeStatus={ nodeStatus }
                actions={ this.props.actions }
              />
            </div>
            <div className={ styles.col4_5 }>
              { this.renderSettings() }
            </div>
          </div>
        </div>
      </Container>
    );
  }

  renderNodeName () {
    const { nodeStatus } = this.props;

    return (
      <span>
        { nodeStatus.nodeName || 'Node' }
      </span>
    );
  }

  renderSettings () {
    const { nodeStatus } = this.props;
    const { rpcSettings, netPeers, netPort = '' } = nodeStatus;
    const peers = `${netPeers.active}/${netPeers.connected}/${netPeers.max}`;

    if (!rpcSettings) {
      return null;
    }

    const rpcPort = rpcSettings.port || '';

    return (
      <div { ...this._test('settings') }>
        <ContainerTitle title='network settings' />
        <Input
          allowCopy
          readOnly
          label='chain'
          value={ nodeStatus.netChain }
          { ...this._test('chain') }
        />
        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label='peers'
              value={ peers }
              { ...this._test('peers') }
            />
          </div>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label='network port'
              value={ netPort.toString() }
              { ...this._test('network-port') }
            />
          </div>
        </div>

        <Input
          allowCopy
          readOnly
          label='rpc enabled'
          value={
            rpcSettings.enabled
              ? 'yes'
              : 'no'
          }
          { ...this._test('rpc-enabled') }
        />
        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label='rpc interface'
              value={ rpcSettings.interface }
              { ...this._test('rpc-interface') }
            />
          </div>
          <div className={ styles.col6 }>
            <Input
              allowCopy
              readOnly
              label='rpc port'
              value={ rpcPort.toString() }
              { ...this._test('rpc-port') }
            />
          </div>
        </div>

        <div className={ styles.row }>
          <div className={ styles.col12 }>
            <Input
              allowCopy
              readOnly
              label='enode'
              value={ nodeStatus.enode }
              { ...this._test('node-enode') }
            />
          </div>
        </div>
      </div>
    );
  }
}
