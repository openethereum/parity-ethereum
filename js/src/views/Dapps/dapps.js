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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import Contracts from '../../contracts';
import { hashToImageUrl } from '../../redux/util';
import { Actionbar, Page } from '../../ui';
import FlatButton from 'material-ui/FlatButton';
import EyeIcon from 'material-ui/svg-icons/image/remove-red-eye';

import { fetchAvailable, fetchManifest } from './registry';
import { readHiddenApps, writeHiddenApps } from './hidden';

import AddDapps from './AddDapps';
import Summary from './Summary';

import styles from './dapps.css';

export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    available: [],
    hidden: [],
    modalOpen: false
  }

  componentDidMount () {
    this.loadAvailableApps();
  }

  render () {
    const { available, hidden, modalOpen } = this.state;
    const apps = available.filter((app) => !hidden.includes(app.id));

    return (
      <div>
        <AddDapps
          available={ available }
          hidden={ hidden }
          open={ modalOpen }
          onHideApp={ this.onHideApp }
          onShowApp={ this.onShowApp }
          onClose={ this.closeModal }
        />
        <Actionbar
          className={ styles.toolbar }
          title='Decentralized Applications'
          buttons={ [
            <FlatButton label='edit' key='edit' icon={ <EyeIcon /> } onClick={ this.openModal } />
          ] }
        />
        <Page>
          <div className={ styles.list }>
            { apps.map(this.renderApp) }
          </div>
        </Page>
      </div>
    );
  }

  renderApp = (app) => {
    if (!app.name) {
      return null;
    }

    return (
      <div
        className={ styles.item }
        key={ app.id }>
        <Summary app={ app } />
      </div>
    );
  }

  onHideApp = (id) => {
    const { hidden } = this.state;
    const newHidden = hidden.concat(id);

    this.setState({ hidden: newHidden });
    writeHiddenApps(newHidden);
  }

  onShowApp = (id) => {
    const { hidden } = this.state;
    const newHidden = hidden.filter((_id) => _id !== id);

    this.setState({ hidden: newHidden });
    writeHiddenApps(newHidden);
  }

  openModal = () => {
    this.setState({ modalOpen: true });
  };

  closeModal = () => {
    this.setState({ modalOpen: false });
  };

  loadAvailableApps () {
    const { api } = this.context;

    fetchAvailable(api)
      .then((available) => {
        this.setState({
          available,
          hidden: readHiddenApps()
        });

        this.loadImages();
        this.loadNetworkApps();
      });
  }

  loadImages () {
    const { available } = this.state;
    const { dappReg } = Contracts.get();

    return Promise
      .all(available.map((app) => dappReg.getImage(app.id)))
      .then((images) => {
        this.setState({
          available: images
            .map(hashToImageUrl)
            .map((image, i) => Object.assign({}, available[i], { image }))
        });
      })
      .catch((error) => {
        console.warn('loadImages', error);
      });
  }

  loadNetworkApps () {
    const { api } = this.context;
    const { available } = this.state;
    const { dappReg } = Contracts.get();
    const networkApps = available.filter((app) => app.network);

    return Promise
      .all(networkApps.map((app) => dappReg.getContent(app.id)))
      .then((content) => {
        return Promise.all(
          networkApps.map((app, index) => {
            const contentHash = api.util.bytesToHex(content[index]);
            console.log(`found content for ${app.id} at ${contentHash}`);

            if (new BigNumber(contentHash).gt(0)) {
              fetchManifest(app, contentHash).then((app) => console.log(app));
            } else {
              return null;
            }
          })
        );
      });
  }

  loadNetworkApp (app, contentHash) {
    return fetchManifest(app, contentHash);
  }
}
