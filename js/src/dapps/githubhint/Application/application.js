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

import React, { Component } from 'react';

import { attachInterface } from '../services';
import Button from '../Button';
import Loading from '../Loading';

import styles from './application.css';

export default class Application extends Component {
  state = {
    loading: true,
    fileHash: false
  }

  componentDidMount () {
    attachInterface()
      .then((accounts, address, accountsInfo, contract, instance, fromAddress) => {
        this.setState({
          accounts,
          address,
          accountsInfo,
          contract,
          instance,
          fromAddress,
          loading: false
        });
      });
  }

  render () {
    const { loading } = this.state;

    return loading
      ? this.renderLoading()
      : this.renderPage();
  }

  renderLoading () {
    return (
      <Loading />
    );
  }

  renderPage () {
    const { fileHash } = this.state;

    return (
      <div className={ styles.container }>
        <div className={ styles.buttons }>
          <Button invert={ fileHash } first onClick={ this.onClickContentHash }>commit</Button>
          <Button invert={ !fileHash } last onClick={ this.onClickFileHash }>file</Button>
        </div>
        <div className={ styles.form }>
          <div className={ styles.box }>
            <div className={ styles.description }>
              Provide a valid GitHub account, repo and { fileHash ? 'filename' : 'commit' } to register. The content information can be used in other contracts that allows for reverse lookups, e.g. image registries, dapp registries, etc.
            </div>
            <div className={ styles.capture }>
              <div>https://github.com/</div>
              <input types='text' placeholder='account/repo' />
            </div>
            <div className={ styles.capture }>
              <div>{ fileHash ? '/' : 'commit #' }</div>
              <input types='text' placeholder={ fileHash ? 'filename' : 'commit hash' } />
            </div>
            <div className={ styles.buttons }>
              <Button onClick={ this.onClickRegister }>register</Button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  onClickContentHash = () => {
    this.setState({ fileHash: false });
  }

  onClickFileHash = () => {
    this.setState({ fileHash: true });
  }
}
