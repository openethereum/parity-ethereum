import React, { Component, PropTypes } from 'react';
import formatNumber from 'format-number';
import bytes from 'bytes';

import { Container } from '../../../../ui';

import styles from './Status.css';
import Value from '../Value';
import MiningSettings from '../MiningSettings';

export default class Status extends Component {
  static propTypes = {
    statusMining: PropTypes.object.isRequired,
    statusSettings: PropTypes.shape({
      chain: PropTypes.string.isRequired,
      networkPort: PropTypes.number.isRequired,
      maxPeers: PropTypes.number.isRequired,
      rpcEnabled: PropTypes.bool.isRequired,
      rpcInterface: PropTypes.string.isRequired,
      rpcPort: PropTypes.number.isRequired
    }).isRequired,
    status: PropTypes.shape({
      name: PropTypes.string,
      version: PropTypes.string.isRequired,
      bestBlock: PropTypes.string.isRequired,
      hashrate: PropTypes.string.isRequired,
      accounts: PropTypes.arrayOf(PropTypes.string).isRequired,
      peers: PropTypes.number.isRequired
    }).isRequired,
    actions: PropTypes.object.isRequired
  }

  render () {
    const { status } = this.props;
    const bestBlock = formatNumber()(status.bestBlock);
    const hashrate = bytes(status.hashrate) || 0;

    return (
      <Container>
        <div className={ styles.container }>
          <div className={ styles.row }>
            <div className={ styles.col3 }>
              <div className={ styles.col12 }>
                <h1><span>Best</span> Block</h1>
                <h1 { ...this._test('best-block') }>#{ bestBlock }</h1>
              </div>
              <div className={ styles.col12 }>
                <h1><span>Hash</span> Rate</h1>
                <h1 { ...this._test('hashrate') }>{ `${hashrate} H/s` }</h1>
              </div>
            </div>
            <div className={ styles.col5 }>
              <MiningSettings
                { ...this._test('mining') }
                statusMining={ this.props.statusMining }
                accounts={ this.props.status.accounts }
                actions={ this.props.actions }
                version={ this.props.status.version }
                />
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
    const { status } = this.props;
    return (
      <span>
        { status.name || 'Node' }
      </span>
    );
  }

  renderSettings () {
    const { status, statusSettings } = this.props;

    return (
      <div { ...this._test('settings') }>
        <h1><span>Network</span> settings</h1>
        <h3>Chain</h3>
        <Value
          value={ statusSettings.chain }
          { ...this._test('chain') }
          />
        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <h3>Peers</h3>
            <Value
              value={ `${status.activePeers}/${status.connectedPeers}/${statusSettings.maxPeers}` }
              { ...this._test('peers') }
              />
          </div>
          <div className={ styles.col6 }>
            <h3>Network port</h3>
            <Value
              value={ statusSettings.networkPort }
              { ...this._test('network-port') }
              />
          </div>
        </div>

        <h3>RPC Enabled</h3>
        <Value
          value={ statusSettings.rpcEnabled ? 'yes' : 'no' }
          { ...this._test('rpc-enabled') }
          />
        <div className={ styles.row }>
          <div className={ styles.col6 }>
            <h3>RPC Interface</h3>
            <Value
              value={ statusSettings.rpcInterface }
              { ...this._test('rpc-interface') }
              />
          </div>
          <div className={ styles.col6 }>
            <h3>RPC Port</h3>
            <Value
              value={ statusSettings.rpcPort }
              { ...this._test('rpc-port') }
              />
          </div>
        </div>
      </div>
    );
  }
}
