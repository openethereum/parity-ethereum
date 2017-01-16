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

import AddressBar from './AddressBar';
import Store from './Store';

import styles from './web.css';

export default class Web extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    params: PropTypes.object.isRequired
  }

  store = Store.get(this.context.api);

  state = {
    displayedUrl: null,
    isLoading: true,
    token: null,
    url: null
  };

  componentDidMount () {
    this.store.generateToken();
    this.store.setUrl(this.props.params.url);
  }

  componentWillReceiveProps (props) {
    this.store.setUrl(props.params.url);
  }

  render () {
    const { displayedUrl, parsedUrl, token, url } = this.store;

    if (!token) {
      return (
        <div className={ styles.wrapper }>
          <h1 className={ styles.loading }>
            Requesting access token...
          </h1>
        </div>
      );
    }

    if (!url) {
      return null;
    }

    const { dappsUrl } = this.context.api;
    const { protocol, host, path } = parsedUrl;
    const address = `${dappsUrl}/web/${token}/${protocol.slice(0, -1)}/${host}${path}`;

    return (
      <div className={ styles.wrapper }>
        <AddressBar
          className={ styles.url }
          isLoading={ isLoading }
          onChange={ this.onUrlChange }
          onRefresh={ this.onRefresh }
          url={ displayedUrl }
        />
        <iframe
          className={ styles.frame }
          frameBorder={ 0 }
          name={ name }
          onLoad={ this.iframeOnLoad }
          sandbox='allow-forms allow-same-origin allow-scripts'
          scrolling='auto'
          src={ address } />
      </div>
    );
  }

  onUrlChange = (url) => {
    if (!hasProtocol.test(url)) {
      url = `https://${url}`;
    }

    store.set(LS_LAST_ADDRESS, url);

    this.setState({
      isLoading: true,
      displayedUrl: url,
      url: url
    });
  };

  onRefresh = () => {
    const { displayedUrl } = this.state;

    // Insert timestamp
    // This is a hack to prevent caching.
    const parsed = parseUrl(displayedUrl);
    parsed.query = parseQuery(parsed.query);
    parsed.query.t = Date.now().toString();
    delete parsed.search;

    this.setState({
      isLoading: true,
      url: formatUrl(parsed)
    });
  };

  iframeOnLoad = () => {
    this.setState({
      isLoading: false
    });
  };
}
