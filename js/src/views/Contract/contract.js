import React, { Component, PropTypes } from 'react';

import Container from '../../ui/Container';

import styles from './style.css';

function nicename (name) {
  return name.split(/(?=[A-Z])/).join(' ');
}

export default class Contract extends Component {
  static contextTypes = {
    api: React.PropTypes.object,
    contracts: PropTypes.array
  }

  static propTypes = {
    params: PropTypes.object
  }

  componentDidMount () {
    this.queryContract();
  }

  render () {
    const contract = this._findContract();

    if (!contract) {
      return null;
    }

    return (
      <div>
        { this.renderQueries(contract) }
        { this.renderFunctions(contract) }
        { this.renderEvents(contract) }
      </div>
    );
  }

  renderEvents (contract) {
    const events = this._findEvents(contract).map((fn) => {
      return (
        <div key={ fn.signature } className={ styles.method }>{ nicename(fn.name) }</div>
      );
    });

    return (
      <Container>
        <h2>events</h2>
        <div className={ styles.methods }>
          { events }
        </div>
      </Container>
    );
  }

  renderFunctions (contract) {
    const functions = this._findFunctions(contract).map((fn) => {
      return (
        <div
          key={ fn.signature }
          className={ styles.method }>
          { nicename(fn.name) }
        </div>
      );
    });

    return (
      <Container>
        <h2>functions</h2>
        <div className={ styles.methods }>
          { functions }
        </div>
      </Container>
    );
  }

  renderQueries (contract) {
    const queries = this._findQueries(contract).map((fn) => {
      return (
        <div
          key={ fn.signature }
          className={ styles.method }>
          { nicename(fn.name) }
        </div>
      );
    });

    return (
      <Container>
        <h2>queries</h2>
        <div className={ styles.methods }>
          { queries }
        </div>
      </Container>
    );
  }

  _sortContracts (a, b) {
    return a.name.localeCompare(b.name);
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

  _findEvents (contract) {
    return !contract
      ? null
      : contract.events.sort(this._sortContracts);
  }

  _findQueries (contract) {
    return !contract
      ? null
      : contract.functions.filter((fn) => fn.constant).sort(this._sortContracts);
  }

  _findFunctions (contract) {
    return !contract
      ? null
      : contract.functions.filter((fn) => !fn.constant).sort(this._sortContracts);
  }

  queryContract = () => {
    const contract = this._findContract();
    const queries = this._findQueries(contract);

    if (!queries) {
      setTimeout(this.queryContract, 5000);
      return;
    }

    const promises = [];

    queries.forEach((query) => {
      if (!query.inputs.length) {
        promises.push(query.call());
      }
    });

    Promise
      .all(promises)
      .then((returns) => {
        console.log(returns);
        setTimeout(this.queryContract, 5000);
      });
  }
}
