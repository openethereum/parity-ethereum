import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

const muiTheme = getMuiTheme(lightBaseTheme);

import registryAbi from '../abi/registry.json';
import gavcoinAbi from '../abi/gavcoin.json';

import Accounts from '../Accounts';
import Events from '../Events';
import Loading from '../Loading';
import Status from '../Status';

const { Api } = window.parity;

const api = new Api(new Api.Transport.Http('/rpc/'));

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    instance: PropTypes.object,
    muiTheme: PropTypes.object
  };

  state = {
    address: null,
    contract: null,
    instance: null,
    loading: true,
    blockNumber: null,
    totalSupply: null,
    remaining: null,
    price: null
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
      <Loading />
    );
  }

  renderInterface () {
    if (this.state.loading) {
      return null;
    }

    return (
      <div>
        <Status
          address={ this.state.address }
          blockNumber={ this.state.blockNumber }
          totalSupply={ this.state.totalSupply }
          remaining={ this.state.remaining }
          price={ this.state.price } />
        <Accounts />
        <Events />
      </div>
    );
  }

  getChildContext () {
    return {
      api,
      instance: this.state.instance,
      muiTheme
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
