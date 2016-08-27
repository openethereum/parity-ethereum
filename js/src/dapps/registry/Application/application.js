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
    owner: null
  }

  componentDidMount () {
    this.attachInterface();
  }

  render () {
    const { address, fee, owner } = this.state;

    if (!address) {
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
            this.setState({
              address,
              fee,
              instance,
              owner
            });
          });
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
