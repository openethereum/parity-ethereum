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

import React, { Component, PropTypes } from 'react';

import Contracts from '../../contracts';
import { fetchAvailable } from '../Dapps/registry';

import styles from './dapp.css';

export default class Dapp extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    params: PropTypes.object
  };

  state = {
    app: null
  }

  componentWillMount () {
    this.lookup();
  }

  render () {
    const { app } = this.state;
    const { dappsUrl } = this.context.api;

    if (!app) {
      return null;
    }

    let src = null;
    if (app.builtin) {
      const dapphost = process.env.NODE_ENV === 'production' && !app.secure
        ? `${dappsUrl}/ui`
        : '';
      src = `${dapphost}/${app.url}.html`;
    } else if (app.local) {
      src = `${dappsUrl}/${app.id}/`;
    } else {
      src = `${dappsUrl}/${app.contentHash}/`;
    }

    return (
      <iframe
        className={ styles.frame }
        frameBorder={ 0 }
        name={ name }
        sandbox='allow-same-origin allow-scripts'
        scrolling='auto'
        src={ src }>
      </iframe>
    );
  }

  lookup () {
    const { api } = this.context;
    const { id } = this.props.params;
    const { dappReg } = Contracts.get();

    fetchAvailable(api)
      .then((available) => {
        return available.find((app) => app.id === id);
      })
      .then((app) => {
        if (app.type !== 'network') {
          return app;
        }

        return dappReg
          .getContent(app.id)
          .then((contentHash) => {
            app.contentHash = contentHash;
            return app;
          });
      })
      .then((app) => {
        this.setState({ app });
      });
  }
}
