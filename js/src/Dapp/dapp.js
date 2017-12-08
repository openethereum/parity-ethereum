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

import React, { Component } from 'react';
import { observer } from 'mobx-react';
import { FormattedMessage } from 'react-intl';
import PropTypes from 'prop-types';

import Api from '@parity/api';
import builtinDapps from '@parity/shared/lib/config/dappsBuiltin.json';
import viewsDapps from '@parity/shared/lib/config/dappsViews.json';
import DappsStore from '@parity/shared/lib/mobx/dappsStore';
import HistoryStore from '@parity/shared/lib/mobx/historyStore';

import styles from './dapp.css';

const internalDapps = []
  .concat(viewsDapps, builtinDapps)
  .map((app) => {
    if (app.id && app.id.substr(0, 2) !== '0x') {
      app.id = Api.util.sha3(app.id);
    }

    return app;
  });

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
  historyStore = HistoryStore.get('dapps');

  componentWillMount () {
    const { id } = this.props.params;

    if (!internalDapps[id] || !internalDapps[id].skipHistory) {
      this.historyStore.add(id);
    }

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
    const { params } = this.props;
    const { app, loading } = this.state;

    if (loading) {
      return null;
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
        src = app.localUrl
          ? `${app.localUrl}?appId=${app.id}`
          : `${dappsUrl}/${app.id}/`;
        break;

      case 'network':
        src = `${dappsUrl}/${app.contentHash}/`;
        break;

      default:
        let dapphost = process.env.DAPPS_URL || (
          process.env.NODE_ENV === 'production'
            ? `${dappsUrl}/ui`
            : ''
        );

        if (dapphost === '/') {
          dapphost = '';
        }

        const appId = this.context.api.util.isHex(app.id)
          ? app.id
          : this.context.api.sha3(app.url);

        src = window.location.protocol === 'file:'
          ? `dapps/${appId}/index.html`
          : `${dapphost}/dapps/${appId}/index.html`;
        break;
    }

    let hash = '';

    if (params.details) {
      hash = `#/${params.details}`;
    }

    return (
      <iframe
        className={ styles.frame }
        frameBorder={ 0 }
        id='dappFrame'
        name={ name }
        onLoad={ this.onDappLoad }
        sandbox='allow-forms allow-popups allow-same-origin allow-scripts allow-top-navigation'
        scrolling='auto'
        src={ `${src}${hash}` }
      />
    );
  }

  onDappLoad = () => {
    const frame = document.getElementById('dappFrame');

    frame.style.opacity = 1;
  }
}
