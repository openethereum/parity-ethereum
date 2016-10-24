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
import { hashToImageUrl } from '../../redux/util';
import { Actionbar, Page } from '../../ui';
import FlatButton from 'material-ui/FlatButton';
import EyeIcon from 'material-ui/svg-icons/image/remove-red-eye';

import fetchAvailable from './available';
import { read as readVisible, write as writeVisible } from './visible';

import AddDapps from './AddDapps';
import Summary from './Summary';

import styles from './dapps.css';

export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    available: [],
    visible: [],
    modalOpen: false
  }

  componentDidMount () {
    fetchAvailable()
    .then((available) => {
      this.setState({ available });
      this.setState({ visible: readVisible() });
      this.loadImages();
    })
    .catch((err) => {
      console.error('error fetching available apps', err);
    });
  }

  render () {
    const { available, visible, modalOpen } = this.state;
    const apps = available.filter((app) => visible.includes(app.id));

    return (
      <div>
        <AddDapps
          available={ available }
          visible={ visible }
          open={ modalOpen }
          onAdd={ this.onAdd }
          onRemove={ this.onRemove }
          onClose={ this.closeModal }
        />
        <Actionbar
          className={ styles.toolbar }
          title='Decentralized Applications'
          buttons={ [
            <FlatButton label='edit' icon={ <EyeIcon /> } onClick={ this.openModal } />
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

    return Promise.all(available.map((app) => dappReg.getImage(app.hash)))
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

  onAdd = (id) => {
    const oldVisible = this.state.visible;
    if (oldVisible.includes(id)) return;
    const newVisible = oldVisible.concat(id);
    this.setState({ visible: newVisible });
    writeVisible(newVisible);
  }

  onRemove = (id) => {
    const oldVisible = this.state.visible;
    if (!oldVisible.includes(id)) return;
    const newVisible = oldVisible.filter((_id) => _id !== id);
    this.setState({ visible: newVisible });
    writeVisible(newVisible);
  }

  openModal = () => {
    this.setState({ modalOpen: true });
  };
  closeModal = () => {
    this.setState({ modalOpen: false });
  };
}
