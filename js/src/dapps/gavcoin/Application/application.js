import React, { Component, PropTypes } from 'react';

import registryAbi from '../abi/registry.json';
import gavcoinAbi from '../abi/gavcoin.json';

const { Api } = window.parity;

const api = new Api(new Api.Transport.Http('/rpc/'));

export default class Application extends Component {
  static childContextTypes = {
    contract: PropTypes.object
  };

  state = {
    address: null,
    contract: null
  }

  componentDidMount () {
    this.attachInterface();
  }

  render () {
    return (
      <div>
        Welcome to GAVcoin, found at { this.state.address }
      </div>
    );
  }

  getChildContext () {
    return {
      contract: this.state.contract
    };
  }

  attachInterface = () => {
    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        const registry = api.newContract(registryAbi, registryAddress).instance;

        return registry.getAddress.call({}, [Api.format.sha3('gavcoin'), 'A']);
      })
      .then((address) => {
        const contract = api.newContract(gavcoinAbi, address);

        this.setState({
          address,
          contract
        });
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
