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
import { observer } from 'mobx-react';
import { FormattedMessage } from 'react-intl';

import DappsStore from '../Dapps/dappsStore';

import styles from './dapp.css';

@observer
export default class Dapp extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    params: PropTypes.object
  };

  state = {
    app: null,
    loading: true
  };

  store = DappsStore.get(this.context.api);

  componentWillMount () {
    const { id } = this.props.params;

    this.loadApp(id);
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.params.id !== this.props.params.id) {
      this.loadApp(nextProps.params.id);
    }
  }

  loadApp (id) {
    this.setState({ loading: true });

    this.store
      .loadApp(id)
      .then((app) => {
        this.setState({ loading: false, app });
      })
      .catch(() => {
        this.setState({ loading: false });
      });
  }

  render () {
    const { dappsUrl } = this.context.api;
    const { app, loading } = this.state;

    if (loading) {
      return (
        <div className={ styles.full }>
          <div className={ styles.text }>
            <FormattedMessage
              id='dapp.loading'
              defaultMessage='Loading'
            />
          </div>
        </div>
      );
    }

    if (!app) {
      return (
        <div className={ styles.full }>
          <div className={ styles.text }>
            <FormattedMessage
              id='dapp.unavailable'
              defaultMessage='The dapp cannot be reached'
            />
          </div>
        </div>
      );
    }

    let src = null;

    switch (app.type) {
      case 'local':
        src = `${dappsUrl}/${app.id}/`;
        break;
      case 'network':
        src = `${dappsUrl}/${app.contentHash}/`;
        break;
      default:
        let dapphost = process.env.DAPPS_URL || (
          process.env.NODE_ENV === 'production' && !app.secure
            ? `${dappsUrl}/ui`
            : ''
        );

        if (dapphost === '/') {
          dapphost = '';
        }

        src = `${dapphost}/${app.url}.html`;
        break;
    }

    return (
      <iframe
        className={ styles.frame }
        frameBorder={ 0 }
        name={ name }
        sandbox='allow-forms allow-popups allow-same-origin allow-scripts'
        scrolling='auto'
        src={ src }
      />
    );
  }
}
