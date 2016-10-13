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

import { sha3 } from '../../api/util/sha3';
import Contracts from '../../contracts';
import { hashToImageUrl } from '../../redux/util';
import { Actionbar, Page } from '../../ui';

import Summary from './Summary';

import styles from './dapps.css';

const APPS = [
  {
    name: 'Token Deployment',
    description: 'Deploy new basic tokens that you are able to send around',
    author: 'Ethcore <admin@ethcore.io>',
    url: 'basiccoin',
    version: '1.0.0'
  },
  {
    name: 'GAVcoin',
    description: 'Manage your GAVcoins, the hottest new property in crypto',
    author: 'Ethcore <admin@ethcore.io>',
    url: 'gavcoin',
    version: '1.0.0'
  },
  {
    name: 'Registry',
    description: 'A global registry of addresses on the network',
    author: 'Ethcore <admin@ethcore.io>',
    url: 'registry',
    version: '1.0.0'
  },
  {
    name: 'Token Registry',
    description: 'A registry of transactable tokens on the network',
    author: 'Ethcore <admin@ethcore.io>',
    url: 'tokenreg',
    version: '1.0.0'
  },
  {
    name: 'Method Registry',
    description: 'A registry of method signatures for lookups on transactions',
    author: 'Ethcore <admin@ethcore.io>',
    url: 'signaturereg',
    version: '1.0.0'
  },
  {
    name: 'GitHub Hint',
    description: 'A mapping of GitHub URLs to hashes for use in contracts as references',
    author: 'Ethcore <admin@ethcore.io>',
    url: 'githubhint',
    version: '1.0.0'
  }
];

APPS.forEach((app) => {
  app.id = sha3(app.url);
  console.log(`dapps ${app.id} -> ${app.url}`);
});

export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    globalApps: APPS,
    localApps: []
  }

  componentDidMount () {
    this.loadLocalApps();
    this.loadImages();
  }

  render () {
    return (
      <div>
        <Actionbar
          title='Decentralized Applications' />
        <Page>
          <div className={ styles.list }>
            { this.renderGlobalApps() }
          </div>
          <div className={ styles.list }>
            { this.renderLocalApps() }
          </div>
        </Page>
      </div>
    );
  }

  renderApp = (app) => {
    return (
      <div
        className={ styles.item }
        key={ app.url }>
        <Summary app={ app } />
      </div>
    );
  }

  renderGlobalApps () {
    const { globalApps } = this.state;

    return globalApps.map(this.renderApp);
  }

  renderLocalApps () {
    const { localApps } = this.state;

    return localApps.map(this.renderApp);
  }

  loadLocalApps () {
    fetch('http://localhost:8080/api/apps', { method: 'GET' })
      .then((response) => response.ok ? response.json() : [])
      .then((_localApps) => {
        const localApps = _localApps
          .filter((app) => !['home', 'status', 'parity', 'wallet'].includes(app.id))
          .map((app) => {
            app.image = `/app/${app.id}/${app.iconUrl}`;
            app.url = app.id;
            app.local = true;
            return app;
          });
        console.log('loadLocalApps', localApps);
        this.setState({ localApps });
      })
      .catch((error) => {
        console.error('loadLocalApps', error);
      });
  }

  loadImages () {
    const { globalApps } = this.state;
    const { dappReg } = Contracts.get();

    Promise
      .all(globalApps.map((app) => dappReg.getImage(app.id)))
      .then((images) => {
        globalApps.forEach((app, index) => {
          app.image = hashToImageUrl(images[index]);
        });
        this.setState({ globalApps });
      })
      .catch((error) => {
        console.error('loadImages', error);
      });
  }
}
