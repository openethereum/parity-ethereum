// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import store from 'store';

import AddressBar from './AddressBar';

import styles from './web.css';

const LS_LAST_ADDRESS = '_parity::webLastAddress';

export default class Web extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    displayedUrl: this.lastAddress(),
    isLoading: true,
    token: null,
    url: this.lastAddress()
  };

  componentDidMount () {
    this.context.api.signer.generateWebProxyAccessToken().then(token => {
      this.setState({ token });
    });
  }

  address () {
    const { dappsUrl } = this.context.api;
    const { url, token } = this.state;
    const path = url.replace(/:/g, '').replace(/\/\//g, '/');

    return `${dappsUrl}/web/${token}/${path}/`;
  }

  lastAddress () {
    return store.get(LS_LAST_ADDRESS) || 'https://mkr.market';
  }

  render () {
    const { displayedUrl, isLoading, token } = this.state;
    const address = this.address();

    if (!token) {
      return (
        <div className={ styles.wrapper }>
          <h1 className={ styles.loading }>
            Requesting access token...
          </h1>
        </div>
      );
    }

    return (
      <div className={ styles.wrapper }>
        <AddressBar
          className={ styles.url }
          isLoading={ isLoading }
          onChange={ this.handleUpdateUrl }
          onRefresh={ this.handleOnRefresh }
          url={ displayedUrl }
        />
        <iframe
          className={ styles.frame }
          frameBorder={ 0 }
          name={ name }
          onLoad={ this.handleIframeLoad }
          sandbox='allow-forms allow-same-origin allow-scripts'
          scrolling='auto'
          src={ address } />
      </div>
    );
  }

  handleUpdateUrl = (url) => {
    store.set(LS_LAST_ADDRESS, url);

    this.setState({
      isLoading: true,
      displayedUrl: url,
      url: url
    });
  };

  handleOnRefresh = (ev) => {
    const { displayedUrl } = this.state;
    const hasQuery = displayedUrl.indexOf('?') > 0;
    const separator = hasQuery ? '&' : '?';

    this.setState({
      isLoading: true,
      url: `${displayedUrl}${separator}t=${Date.now()}`
    });
  };

  handleIframeLoad = (ev) => {
    this.setState({
      isLoading: false
    });
  };
}

