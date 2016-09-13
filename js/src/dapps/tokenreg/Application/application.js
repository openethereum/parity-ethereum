import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

import registryAbi from '../abi/registry.json';
import tokenregAbi from '../abi/tokenreg.json';
import Loading from '../Loading';
import Status from '../Status';

const { api } = window.parity;

const muiTheme = getMuiTheme(lightBaseTheme);

export default class Application extends Component {
  static childContextTypes = {
    instance: PropTypes.object,
    muiTheme: PropTypes.object
  }

  state = {
    address: null,
    instance: null,
    loading: true
  }

  componentDidMount () {
    this.attachInterface();
  }

  render () {
    const { address, fee, loading, owner } = this.state;

    if (loading) {
      return (
        <Loading />
      );
    }

    return (
      <div>
        <Status
          address={ address }
          fee={ fee }
          owner={ owner } />
      </div>
    );
  }

  getChildContext () {
    const { instance } = this.state;

    return {
      instance,
      muiTheme
    };
  }

  onNewBlockNumber = (_error, blockNumber) => {
    const { instance } = this.state;

    if (_error) {
      console.error('onNewBlockNumber', _error);
      return;
    }

    instance.fee
      .call()
      .then((fee) => {
        this.setState({
          fee
        });
      });
  }

  attachInterface = () => {
    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        console.log(`registry found at ${registryAddress}`);
        const registry = api.newContract(registryAbi, registryAddress).instance;

        return registry.getAddress.call({}, [api.format.sha3('tokenreg'), 'A']);
      })
      .then((address) => {
        console.log(`tokenreg was found at ${address}`);
        const { instance } = api.newContract(tokenregAbi, address);

        return Promise
          .all([
            instance.owner.call(),
            instance.fee.call()
          ])
          .then(([owner, fee]) => {
            console.log(`owner as ${owner}, fee set at ${fee.toFormat()}`);
            this.setState({
              address,
              fee,
              instance,
              loading: false,
              owner
            });

            api.subscribe('eth_blockNumber', this.onNewBlockNumber);
          });
      })
      .catch((error) => {
        console.error('attachInterface', error);
      });
  }
}
