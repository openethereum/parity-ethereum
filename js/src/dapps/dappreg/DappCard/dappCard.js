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

import keycode from 'keycode';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';

import DappModal from '../DappModal';

import styles from './dappCard.css';

export default class DappCard extends Component {
  static propTypes = {
    dapp: PropTypes.object.isRequired
  };

  state = {
    open: false
  };

  render () {
    const { dapp } = this.props;
    const { id, imageUrl, manifest } = dapp;

    return (
      <div>
        { this.renderModal() }

        <div
          className={ styles.card }
          onClick={ this.handleOpen }
          onKeyPress={ this.handleKeyPress }
          ref='card'
          tabIndex={ 0 }
        >
          <div className={ styles.icon }>
            { this.renderImage(imageUrl) }
          </div>

          <span
            className={ styles.name }
            title={ id }
          >
            { manifest && manifest.name || id }
          </span>

          { this.renderVersion(manifest) }
          { this.renderAuthor(manifest) }
        </div>
      </div>
    );
  }

  renderModal () {
    const { dapp } = this.props;
    const { open } = this.state;

    return (
      <DappModal
        dapp={ dapp }
        onClose={ this.handleClose }
        open={ open }
      />
    );
  }

  renderImage (url) {
    return (
      <img src={ url } />
    );
  }

  renderVersion (manifest) {
    if (!manifest || !manifest.version) {
      return null;
    }

    return (
      <span className={ styles.version }>
        v{ manifest.version }
      </span>
    );
  }

  renderAuthor (manifest) {
    if (!manifest || !manifest.author) {
      return null;
    }

    return (
      <span className={ styles.author }>
        by { manifest && manifest.author }
      </span>
    );
  }

  handleKeyPress = (event) => {
    const codeName = keycode(event);

    if (codeName === 'enter') {
      return this.handleOpen();
    }

    return event;
  }

  handleClose = () => {
    this.setState({ open: false }, () => {
      if (!this.refs.card) {
        return false;
      }

      setTimeout(() => {
        const element = ReactDOM.findDOMNode(this.refs.card);

        element && element.focus();
      }, 50);
    });
  }

  handleOpen = () => {
    this.setState({ open: true });
  }
}
