import React, { Component, PropTypes } from 'react';

import Summary from './Summary';

import styles from './contracts.css';

export default class Contracts extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    contracts: PropTypes.array.isRequired
  }

  state = {
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
    if (!this.context.contracts) {
      return null;
    }

    return this.context.contracts.map((contract, idx) => {
      return (
        <div
          className={ styles.contract }
          key={ contract.address }>
          <Summary
            contract={ contract } />
        </div>
      );
    });
  }
}
