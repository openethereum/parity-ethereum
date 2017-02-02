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

import React, { Component, PropTypes } from 'react';
import store from 'store';
import { parse as parseUrl, format as formatUrl } from 'url';
import { parse as parseQuery } from 'querystring';

import { Button, Modal } from '~/ui';
import { CancelIcon, CheckIcon } from '~/ui/Icons';
import AddressBar from './AddressBar';

import { EXTENSION_PAGE, shouldShowWarning, installExtension, hideWarning } from './extension-warning';
import styles from './web.css';

const LS_LAST_ADDRESS = '_parity::webLastAddress';

const hasProtocol = /^https?:\/\//;

export default class Web extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    params: PropTypes.object.isRequired
  }

  state = {
    displayedUrl: null,
    isLoading: true,
    token: null,
    url: null,
    extensionWarningShown: false
  };

  componentDidMount () {
    const { api } = this.context;
    const { params } = this.props;

    api
      .signer
      .generateWebProxyAccessToken()
      .then((token) => {
        this.setState({ token });
      });

    this.setUrl(params.url);

    if (shouldShowWarning()) {
      this.setState({
        extensionWarningShown: true
      });
    }
  }

  componentWillReceiveProps (props) {
    this.setUrl(props.params.url);
  }

  setUrl = (url) => {
    url = url || store.get(LS_LAST_ADDRESS) || 'https://mkr.market';
    if (!hasProtocol.test(url)) {
      url = `https://${url}`;
    }

    this.setState({ url, displayedUrl: url });
  };

  render () {
    const { displayedUrl, isLoading, token } = this.state;

    if (!token) {
      return (
        <div className={ styles.wrapper }>
          <h1 className={ styles.loading }>
            Requesting access token...
          </h1>
        </div>
      );
    }

    const { dappsUrl } = this.context.api;
    const { url, extensionWarningShown } = this.state;

    if (!url || !token) {
      return null;
    }

    const parsed = parseUrl(url);
    const { protocol, host, path } = parsed;
    const address = `${dappsUrl}/web/${token}/${protocol.slice(0, -1)}/${host}${path}`;

    return (
      <div className={ styles.wrapper }>
        { extensionWarningShown ? this.renderExtensionWarning() : null }
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
          src={ address }
        />
      </div>
    );
  }

  renderExtensionWarning () {
    const cancel = (
      <Button
        icon={ <CancelIcon /> }
        key='close'
        label='No Thanks'
        onClick={ this.hideExtensionWarning }
      />
    );
    const install = (
      <Button
        icon={ <CheckIcon /> }
        key='install'
        label='Install'
        onClick={ this.openExtensionPage }
      />
    );

    return (
      <Modal
        actions={ [ cancel, install ] }
        title='Install the Parity Extension'
        visible
      >
        <p>Parity now has a Chrome extension. We strongly recommend to install it.</p>
      </Modal>
    );
  }

  hideExtensionWarning = () => {
    hideWarning();
    this.setState({
      extensionWarningShown: false
    });
  }

  openExtensionPage = () => {
    installExtension()
      .then(hideWarning)
      .catch((err) => {
        console.error(err);
        window.open(EXTENSION_PAGE, '_blank');
      });
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
