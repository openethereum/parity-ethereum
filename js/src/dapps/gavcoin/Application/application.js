import React, { Component, PropTypes } from 'react';

import registryAbi from '../abi/registry.json';
import gavcoinAbi from '../abi/gavcoin.json';

const { Api } = window.parity;

const api = new Api(new Api.Transport.Http('/rpc/'));

export default class Application extends Component {
  static childContextTypes = {
    instance: PropTypes.object
  };

  state = {
    address: null,
    contract: null,
    instance: null,
    loading: true,
    blockNumber: 0,
    totalSupply: 0,
    remaining: 0,
    price: 0
  }

  componentDidMount () {
    this.attachInterface();
  }

  render () {
    return (
      <div>
        { this.renderLoading() }
        { this.renderInterface() }
      </div>
    );
  }

  renderLoading () {
    if (!this.state.loading) {
      return null;
    }

    return (
      <div>GAVcoin is loading ...</div>
    );
  }

  renderInterface () {
    if (this.state.loading) {
      return null;
    }

    return (
      <div>
        <div>Welcome to GAVcoin, found at { this.state.address }</div>
        { this.renderStatus() }
      </div>
    );
  }

  renderStatus () {
    if (!this.state.blockNumber) {
      return null;
    }

    return (
      <div>#{ this.state.blockNumber }: { this.state.remaining } coins remaining ({ this.state.totalSupply } total), price of { this.state.price }</div>
    );
  }

  getChildContext () {
    return {
      instance: this.state.instance
    };
  }

  onNewBlockNumber = (blockNumber) => {
    const { instance } = this.state;

    Promise
      .all([
        instance.totalSupply.call(),
        instance.remaining.call(),
        instance.price.call()
      ])
      .then(([totalSupply, remaining, price]) => {
        this.setState({
          blockNumber: blockNumber.toFormat(),
          totalSupply: totalSupply.toFormat(),
          remaining: remaining.toFormat(),
          price: price.div(1000000).toFormat()
        });
      });
  }

  attachInterface = () => {
    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        console.log(`the registry was found at ${registryAddress}`);

        const registry = api.newContract(registryAbi, registryAddress).instance;

        return registry.getAddress.call({}, [Api.format.sha3('gavcoin'), 'A']);
      })
      .then((address) => {
        console.log(`gavcoin was found at ${address}`);

        const contract = api.newContract(gavcoinAbi, address);
        const instance = contract.instance;

        this.setState({
          address,
          contract,
          instance,
          loading: false
        });

        api.events.subscribe('eth.blockNumber', this.onNewBlockNumber);
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
