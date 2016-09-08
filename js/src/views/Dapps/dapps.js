import React, { Component, PropTypes } from 'react';

import { Actionbar, Page } from '../../ui';

import Summary from './Summary';

import styles from './dapps.css';

export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    apps: [
      {
        name: 'GAVcoin',
        address: '0x6C5b287A875298f773225e72ce3fA8B2782e0347',
        description: 'Manage your GAVcoins, the hottest new property in crypto',
        url: 'gavcoin'
      },
      {
        name: 'Registry',
        address: '0x8E4e9B13D4b45Cb0befC93c3061b1408f67316B2',
        description: 'A global registry of addresses on the network',
        url: 'registry'
      },
      {
        name: 'Token Registry',
        address: '0x1AE76cf6Ee3955F773C429801a203f08c84B7cc5',
        description: 'A registry of transactable tokens on the network',
        url: 'tokenreg'
      }
    ]
  }

  render () {
    return (
      <div>
        <Actionbar
          title='Decentralized Applications' />
        <Page>
          <div className={ styles.contracts }>
            { this.renderApps() }
          </div>
        </Page>
      </div>
    );
  }

  renderApps () {
    const { apps } = this.state;

    return apps.map((app, idx) => {
      return (
        <div
          className={ styles.contract }
          key={ app.address }>
          <Summary
            app={ app } />
        </div>
      );
    });
  }
}
