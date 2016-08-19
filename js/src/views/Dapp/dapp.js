import React, { Component, PropTypes } from 'react';

import Container from '../../ui/Container';

import styles from './style.css';

export default class Dapp extends Component {
  static contextTypes = {
    api: React.PropTypes.object,
    contracts: PropTypes.array
  }

  static propTypes = {
    params: PropTypes.object
  }

  render () {
    const contract = this._findContract();
    const sort = (a, b) => a.name.localeCompare(b.name);
    const nicename = (name) => name.split(/(?=[A-Z])/).join(' ');

    if (!contract) {
      return null;
    }

    const functions = contract.functions
      .filter((fn) => !fn.constant)
      .sort(sort)
      .map((fn) => {
        return (
          <div className={ styles.method }>{ nicename(fn.name) }</div>
        );
      });

    const queries = contract.functions
      .filter((fn) => fn.constant)
      .sort(sort)
      .map((fn) => {
        return (
          <div className={ styles.method }>{ nicename(fn.name) }</div>
        );
      });

    const events = contract.events
      .sort(sort)
      .map((fn) => {
        return (
          <div className={ styles.method }>{ nicename(fn.name) }</div>
        );
      });

    return (
      <div>
        <Container>
          <h2>queries</h2>
          <div className={ styles.methods }>
            { queries }
          </div>
        </Container>
        <Container>
          <h2>functions</h2>
          <div className={ styles.methods }>
            { functions }
          </div>
        </Container>
        <Container>
          <h2>events</h2>
          <div className={ styles.methods }>
            { events }
          </div>
        </Container>
      </div>
    );
  }

  _findContract () {
    if (!this.props.params.address || !this.context.contracts) {
      return null;
    }

    const address = this.props.params.address.toLowerCase();
    const contract = this.context.contracts.find((c) => c.address.toLowerCase() === address);

    return !contract
      ? null
      : contract.contract;
  }
}
