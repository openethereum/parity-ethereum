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

import styles from './web.css';

export default class Web extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    token: null,
    isLoading: true,
    displayedUrl: 'https://mkr.market',
    url: 'https://mkr.market'
  };

  handleUpdateUrl = (url) => {
    this.setState({
      isLoading: true,
      displayedUrl: url,
      url: url
    });
  };

  handleOnRefresh = (ev) => {
    // TODO [ToDr]
    this.setState({
      isLoading: true,
      url: `${this.state.displayedUrl}?t=${Date.now()}`
    });
  };

  handleIframeLoad = (ev) => {
    this.setState({
      isLoading: false
    });
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

  render () {
    const { displayedUrl, isLoading, token } = this.state;
    const address = this.address();

    if (!token) {
      return (
        <div
          className={ styles.wrapper }
          >
          <h1 className={ styles.loading }>
            Requesting access token...
          </h1>
        </div>
      );
    }

    return (
      <div
        className={ styles.wrapper }
        >
        <AddressBar
          url={ displayedUrl }
          isLoading={ isLoading }
          onChange={ this.handleUpdateUrl }
          onRefresh={ this.handleOnRefresh }
        />
        <iframe
          className={ styles.frame }
          frameBorder={ 0 }
          name={ name }
          sandbox='allow-forms allow-same-origin allow-scripts'
          scrolling='auto'
          onLoad={ this.handleIframeLoad }
          src={ address } />
      </div>
    );
  }
}

class AddressBar extends Component {

  static propTypes = {
    isLoading: PropTypes.bool.isRequired,
    url: PropTypes.string.isRequired,
    onChange: PropTypes.func.isRequired,
    onRefresh: PropTypes.func.isRequired
  };

  state = {
    currentUrl: this.props.url
  };

  onUpdateUrl = (ev) => {
    this.setState({
      currentUrl: ev.target.value
    });
  };

  onKey = (ev) => {
    const KEY_ESC = 27;
    const KEY_ENTER = 13;

    const key = ev.which;

    if (key === KEY_ESC) {
      this.setState({
        currentUrl: this.props.url
      });
      return;
    }

    if (key === KEY_ENTER) {
      this.onGo();
      return;
    }
  };

  onGo = () => {
    if (this.isPristine()) {
      this.props.onRefresh();
    } else {
      this.props.onChange(this.state.currentUrl);
    }
  };

  isPristine () {
    return this.state.currentUrl === this.props.url;
  }

  componentWillReceiveProps (nextProps) {
    if (this.props.url === nextProps.url) {
      return;
    }

    this.setState({
      currentUrl: nextProps.url
    });
  }

  render () {
    const { isLoading } = this.props;
    const { currentUrl } = this.state;
    const isPristine = this.isPristine();

    return (
      <div className={ styles.url }>
        <button
          disabled={ isLoading }
          onClick={ this.onGo }
          >
          { isLoading ? 'Loading' : 'Refresh'}
        </button>
        <input
          onChange={ this.onUpdateUrl }
          onKeyDown={ this.onKey }
          type='text'
          value={ currentUrl }
        />
        <button
          disabled={ isPristine }
          onClick={ this.onGo }
          >
          Go
        </button>
      </div>
    );
  }

}
