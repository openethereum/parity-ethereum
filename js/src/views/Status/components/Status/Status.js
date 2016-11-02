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
import bytes from 'bytes';

import { Container, ContainerTitle, Input } from '../../../../ui';

import styles from './Status.css';
import MiningSettings from '../MiningSettings';

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
              <div className={ styles.col12 }>
                <ContainerTitle title='best block' />
                <h2 { ...this._test('best-block') } className={ styles.blockinfo }>
                  #{ nodeStatus.blockNumber.toFormat() }
                </h2>
              </div>
              <div className={ styles.col12 }>
                <ContainerTitle title='peers' />
                <h2 { ...this._test('peers') } className={ styles.blockinfo }>
                  { peers }
                </h2>
              </div>
              <div className={ styles.col12 }>
                <ContainerTitle title='hash rate' />
                <h2 { ...this._test('hashrate') } className={ styles.blockinfo }>
                  { `${hashrate} H/s` }
                </h2>
              </div>
            </div>
            <div className={ styles.col5 }>
              <MiningSettings
                { ...this._test('mining') }
                nodeStatus={ nodeStatus }
                actions={ this.props.actions } />
            </div>
            <div className={ styles.col4 }>
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
    const { rpcSettings, netPeers } = nodeStatus;
    const peers = `${netPeers.active}/${netPeers.connected}/${netPeers.max}`;

    return (
      <div { ...this._test('settings') }>
        <ContainerTitle title='network settings' />
        <Input
          readOnly
          label='chain'
          value={ nodeStatus.netChain }
          { ...this._test('chain') } />
        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <Input
              readOnly
              label='peers'
              value={ peers }
              { ...this._test('peers') } />
          </div>
          <div className={ styles.col6 }>
            <Input
              readOnly
              label='network port'
              value={ nodeStatus.netPort.toString() }
              { ...this._test('network-port') } />
          </div>
        </div>

        <Input
          readOnly
          label='rpc enabled'
          value={ rpcSettings.enabled ? 'yes' : 'no' }
          { ...this._test('rpc-enabled') } />
        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <Input
              readOnly
              label='rpc interface'
              value={ rpcSettings.interface }
              { ...this._test('rpc-interface') } />
          </div>
          <div className={ styles.col6 }>
            <Input
              readOnly
              label='rpc port'
              value={ rpcSettings.port.toString() }
              { ...this._test('rpc-port') } />
          </div>
        </div>
      </div>
    );
  }
}
