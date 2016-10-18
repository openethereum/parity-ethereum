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

import fetchAvailable from './available';
import { read as readVisible } from './visible';

import Summary from './Summary';

import styles from './dapps.css';

export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    available: [],
    visible: []
  }

  componentDidMount () {
    fetchAvailable()
    .then((available) => {
      this.setState({ available })
      this.setState({ visible: readVisible() });
      this.loadImages();
    })
    .catch((err) => {
      console.error('error fetching available apps', err);
    });
  }

  render () {
    const { available, visible } = this.state;
    const apps = available.filter((app) => visible.includes(app.id))

    return (
      <div>
        <Actionbar
          title='Decentralized Applications' />
        <Page>
          <div className={ styles.list }>
            { apps.map(this.renderApp) }
          </div>
        </Page>
      </div>
    );
  }

  renderApp = (app) => {
    return (
      <div
        className={ styles.item }
        key={ app.id }>
        <Summary app={ app } />
      </div>
    );
  }

  loadImages () {
    const { available } = this.state;
    const { dappReg } = Contracts.get();

    return Promise.all(available.map((app) => dappReg.getImage(sha3(app.id))))
    .then((images) => {
      this.setState({
        available: images
          .map(hashToImageUrl)
          .map((image, i) => Object.assign({}, available[i], { image }))
      });
    })
    .catch((err) => {
      console.error('error loading dapp images', err);
    });
  }
}
