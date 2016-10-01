// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';

import * as abis from '../../../contracts/abi';
import { api } from '../parity';

import Header from './Header';
import Loading from './Loading';
import PAGES from './pages';

import styles from './application.css';

export default class Application extends Component {
  static childContextTypes = {
    managerInstance: PropTypes.object,
    registryInstance: PropTypes.object,
    tokenregInstance: PropTypes.object
  }

  static propTypes = {
    children: PropTypes.node.isRequired
  }

  state = {
    loading: true,
    managerInstance: null,
    registryInstance: null,
    tokenregInstance: null
  }

  componentDidMount () {
    this.attachInstance();
  }

  render () {
    const { children } = this.props;
    const { loading } = this.state;

    if (loading) {
      return (
        <Loading />
      );
    }

    const path = (window.location.hash || '').split('?')[0].split('/')[1];
    const page = PAGES.find((page) => page.path === path);
    const style = { background: page.color };

    return (
      <div className={ styles.container } style={ style }>
        <Header />
        <div className={ styles.body }>
          { children }
        </div>
      </div>
    );
  }

  getChildContext () {
    const { managerInstance, registryInstance, tokenregInstance } = this.state;

    return {
      managerInstance,
      registryInstance,
      tokenregInstance
    };
  }

  attachInstance () {
    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        console.log(`contract was found at registry=${registryAddress}`);

        const registry = api.newContract(abis.registry, registryAddress).instance;

        return Promise
          .all([
            registry.getAddress.call({}, [api.util.sha3('basiccoinmanager'), 'A']),
            registry.getAddress.call({}, [api.util.sha3('basiccoinregistry'), 'A']),
            registry.getAddress.call({}, [api.util.sha3('tokenreg'), 'A'])
          ]);
      })
      .then(([managerAddress, registryAddress, tokenregAddress]) => {
        console.log(`contracts were found at manager=${managerAddress}, registry=${registryAddress}, tokenreg=${registryAddress}`);

        const managerInstance = api.newContract(abis.basiccoinmanager, managerAddress).instance;
        const registryInstance = api.newContract(abis.tokenreg, registryAddress).instance;
        const tokenregInstance = api.newContract(abis.tokenreg, tokenregAddress).instance;

        this.setState({
          loading: false,
          managerInstance,
          registryInstance,
          tokenregInstance
        });
      });
  }
}
