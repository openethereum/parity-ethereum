// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import BigNumber from 'bignumber.js';
import React, { Component } from 'react';

import { attachInterface, attachBlockNumber } from '../services';
import Button from '../Button';
import Events from '../Events';
import Header from '../Header';
import Import from '../Import';
import Loading from '../Loading';

import styles from './application.css';

export default class Application extends Component {
  state = {
    accounts: {},
    address: null,
    accountsInfo: {},
    blockNumber: new BigNumber(0),
    contract: null,
    instance: null,
    loading: true,
    totalSignatures: new BigNumber(0),
    showImport: false
  }

  componentDidMount () {
    return attachInterface()
      .then((state) => {
        this.setState(Object.assign({}, state, { loading: false }));

        return attachBlockNumber(state.instance, (state) => {
          this.setState(state);
        });
      })
      .catch((error) => {
        console.error('componentDidMount', error);
      });
  }

  render () {
    const { loading } = this.state;

    if (loading) {
      return (
        <Loading />
      );
    }

    return (
      <div className={ styles.container }>
        { this.renderHeader() }
        { this.renderImport() }
        { this.renderEvents() }
      </div>
    );
  }

  renderHeader () {
    const { blockNumber, totalSignatures } = this.state;

    return (
      <Header
        blockNumber={ blockNumber }
        totalSignatures={ totalSignatures }
      />
    );
  }

  renderImport () {
    const { instance, showImport } = this.state;

    if (showImport) {
      return (
        <Import
          instance={ instance }
          visible={ showImport }
          onClose={ this.toggleImport }
        />
      );
    }

    return (
      <div className={ styles.actions }>
        <Button invert onClick={ this.toggleImport }>import abi</Button>
      </div>
    );
  }

  renderEvents () {
    const { accountsInfo, contract } = this.state;

    return (
      <Events
        accountsInfo={ accountsInfo }
        contract={ contract }
      />
    );
  }

  toggleImport = () => {
    this.setState({
      showImport: !this.state.showImport
    });
  }
}
