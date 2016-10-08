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

import { api } from '../parity';
import { attachInterface } from '../services';
import Button from '../Button';
import Loading from '../Loading';

import styles from './application.css';

const INVALID_URL_HASH = '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470';

export default class Application extends Component {
  state = {
    loading: true,
    url: '',
    urlError: null,
    contentHash: null,
    contentHashError: null
  }

  componentDidMount () {
    attachInterface()
      .then((state) => {
        this.setState(state, () => {
          this.setState({ loading: false });
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
    const { url, urlError, contentHash, contentHashError } = this.state;

    return (
      <div className={ styles.container }>
        <div className={ styles.form }>
          <div className={ styles.box }>
            <div className={ styles.description }>
              Provide a valid URL to register. The content information can be used in other contracts that allows for reverse lookups, e.g. image registries, dapp registries, etc.
            </div>
            <div className={ styles.capture }>
              <input
                type='text'
                placeholder='http://domain/filename'
                value={ url }
                className={ urlError ? styles.error : null }
                onChange={ this.onChangeUrl } />
            </div>
            <div className={ contentHashError ? styles.hashError : styles.hashOk }>
              { contentHashError || contentHash }
            </div>
            <div className={ styles.buttons }>
              <Button
                onClick={ this.onClickRegister }
                disabled={ !!contentHashError || !!urlError || url.length === 0 }>register</Button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  onClickContentHash = () => {
    this.setState({ fileHash: false, commit: '' });
  }

  onClickFileHash = () => {
    this.setState({ fileHash: true, commit: 0 });
  }

  onChangeUrl = (event) => {
    const url = event.target.value;
    let urlError = null;

    if (url && url.length) {
      var re = /^https?:\/\/(?:www\.|(?!www))[^\s\.]+\.[^\s]{2,}/g;
      urlError = re.test(url)
        ? null
        : 'not matching rexex';
    }

    this.setState({ url, urlError, contentHashError: 'hash lookup in progress' }, () => {
      this.lookupHash();
    });
  }

  onClickRegister = (event) => {
  }

  lookupHash () {
    const { url } = this.state;

    api.ethcore
      .hashContent(url)
      .then((contentHash) => {
        console.log('lookupHash', contentHash);
        if (contentHash === INVALID_URL_HASH) {
          this.setState({ contentHashError: 'invalid url endpoint', contentHash });
        } else {
          this.setState({ contentHashError: null, contentHash });
        }
      })
      .catch((error) => {
        console.error('lookupHash', error);
        this.setState({ contentHashError: error.message, contentHash: null });
      });
  }
}
