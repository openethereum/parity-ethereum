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

import styles from './dappCard.css';

export default class DappCard extends Component {
  static propTypes = {
    dapp: PropTypes.object.isRequired
  };

  render () {
    const { dapp } = this.props;
    const { id, imageUrl, manifest } = dapp;

    return (
      <div className={ styles.card }>
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
}
