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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import AddressBar from './AddressBar';
import Store from './store';

import styles from './web.css';

@observer
export default class Web extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    params: PropTypes.object.isRequired
  }

  store = Store.get(this.context.api);

  state = {
    isWarningDismissed: false
  }

  componentDidMount () {
    this.store.gotoUrl(this.props.params.url);
  }

  componentWillReceiveProps (props) {
    this.store.gotoUrl(props.params.url);
  }

  render () {
    const { currentUrl, token } = this.store;

    if (!token) {
      return (
        <div className={ styles.wrapper }>
          <h1 className={ styles.loading }>
            <FormattedMessage
              id='web.requestToken'
              defaultMessage='Requesting access token...'
            />
          </h1>
        </div>
      );
    }

    return currentUrl
      ? this.renderFrame()
      : null;
  }

  renderFrame () {
    const { encodedPath, frameId } = this.store;

    return (
      <div className={ styles.wrapper }>
        <AddressBar
          className={ styles.url }
          store={ this.store }
        />
        <iframe
          className={ styles.frame }
          frameBorder={ 0 }
          id={ frameId }
          name={ frameId }
          onLoad={ this.iframeOnLoad }
          sandbox='allow-forms allow-same-origin allow-scripts'
          scrolling='auto'
          src={ encodedPath }
        />
        { this.renderWarning() }
      </div>
    );
  }

  renderWarning () {
    if (this.state.isWarningDismissed) {
      return null;
    }

    return (
      <div className={ styles.warning }>
        <p>
          WARNING: The web browser dapp is not safe as a general purpose browser.
          Make sure to only visit web3-enabled sites that you trust.
          Do not use it to browse web2.0 and never log in to any service - web3 dapps should not require that.
        </p>
        <div className={ styles.warningClose }>
          <a onClick={ this.dismissWarning }>Okay!</a>
        </div>
      </div>
    );
  }

  dismissWarning = () => {
    this.setState({
      isWarningDismissed: true
    });
  };

  iframeOnLoad = () => {
    this.store.setLoading(false);
  };
}
