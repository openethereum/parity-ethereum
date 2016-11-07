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
import { observer } from 'mobx-react';

import DappsStore from '../Dapps/dappsStore';

import styles from './dapp.css';

@observer
export default class Dapp extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    params: PropTypes.object
  };

  store = new DappsStore(this.context.api);

  render () {
    const { dappsUrl } = this.context.api;
    const { id } = this.props.params;
    const app = this.store.apps.find((app) => app.id === id);

    if (!app) {
      return null;
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
        const dapphost = process.env.NODE_ENV === 'production' && !app.secure
          ? `${dappsUrl}/ui`
          : '';
        src = `${dapphost}/${app.url}.html`;
        break;
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
}
