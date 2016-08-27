import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

const muiTheme = getMuiTheme(lightBaseTheme);

import registryAbi from '../abi/registry.json';
import Loading from '../Loading';
import Status from '../Status';

const { Api } = window.parity;

const api = new Api(new Api.Transport.Http('/rpc/'));

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    instance: PropTypes.object,
    muiTheme: PropTypes.object
  }

  state = {
    address: null,
    fee: null,
    instance: null,
    loading: true,
    owner: null
  }

  componentDidMount () {
    this.attachInterface();
  }

  render () {
    const { address, fee, loading, owner } = this.state;

    if (!loading) {
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
    return {
      api,
      instance: this.state.instance,
      muiTheme
    };
  }

  onNewBlockNumber = (blockNumber) => {
    const { instance } = this.state;

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
      .then((address) => {
        const { instance } = api.newContract(registryAbi, address);

        return Promise
          .all([
            instance.owner.call(),
            instance.fee.call()
          ])
          .then(([owner, fee]) => {
            api.events.subscribe('eth.blockNumber', this.onNewBlockNumber);

            this.setState({
              address,
              fee,
              instance,
              loading: false,
              owner
            });
          });
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
