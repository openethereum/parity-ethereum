import React, { Component, PropTypes } from 'react';

import Summary from './Summary';

import styles from './style.css';

export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  state = {
    apps: [
      {
        name: 'GAVcoin',
        address: '0x6C5b287A875298f773225e72ce3fA8B2782e0347',
        description: 'Mnage your GAVcoins, the hottest new property in crypto',
        url: '/dapps/gavcoin.html'
      }
    ]
  }

  render () {
    return (
      <div>
        <div className={ styles.contracts }>
          { this.renderContracts() }
        </div>
      </div>
    );
  }

  renderContracts () {
    return this.state.apps.map((app, idx) => {
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
