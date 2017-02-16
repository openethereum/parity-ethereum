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

import styles from './dappModal.css';

export default class DappCard extends Component {
  static propTypes = {
    dapp: PropTypes.object.isRequired,
    open: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired
  };

  render () {
    const { dapp, open } = this.props;
    const { id } = dapp;

    const classes = [ styles.modal ];

    if (open) {
      classes.push(styles.open);
    }

    return (
      <div className={ classes.join(' ') }>
        <div className={ styles.container }>
          <div
            className={ styles.close }
            onClick={ this.handleClose }
            title='close'
          >
            ❌
          </div>

          { this.renderHeader(dapp) }
          { this.renderContent(dapp) }
        </div>
      </div>
    );
  }

  renderContent (dapp) {
    const manifest = dapp.manifest || {};

    return (
      <div className={ styles.content }>
        <div className={ styles.code }>
          <div className={ styles.codeTitle }>manifest.json</div>
          <div className={ styles.codeContainer }>
            <code>{ JSON.stringify(manifest, null, 2) }</code>
          </div>
        </div>
      </div>
    );
  }

  renderHeader (dapp) {
    const { id, imageUrl } = dapp;
    const manifest = dapp.manifest || {};

    const infos = [];

    if (manifest.version) {
      infos.push(`v${manifest.version}`);
    }

    if (manifest.author) {
      infos.push(`by ${manifest.author}`);
    }

    return (
      <div className={ styles.header }>
        <div className={ styles.icon }>
          <img src={ imageUrl } />
        </div>
        <div>
          <div className={ styles.name }>
            { manifest.name || 'Unnamed' }
          </div>
          <div className={ styles.info }>
            { id }
          </div>
          <div className={ styles.info }>
            { infos.length > 0 ? infos.join(', ') : null }
          </div>
        </div>
      </div>
    );
  }

  handleClose = () => {
    this.props.onClose();
  }
}
