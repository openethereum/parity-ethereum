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

import ModalRegister from '../ModalRegister';

import PlusImage from '~/../assets/images/dapps/plus.svg';
import dappCardStyles from '../DappCard/dappCard.css';

export default class CreateDappCard extends Component {
  state = {
    dappId: null,
    open: false
  };

  render () {
    return (
      <div className={ dappCardStyles.container }>
        { this.renderModal() }

        <div
          className={ [ dappCardStyles.card, dappCardStyles.register ].join(' ') }
          onClick={ this.handleOpen }
          tabIndex={ 0 }
        >
          <div className={ dappCardStyles.icon }>
            <img src={ PlusImage } />
          </div>

          <span className={ dappCardStyles.name }>
            Register a dapp
          </span>
        </div>
      </div>
    );
  }

  renderModal () {
    const { dappId, open } = this.state;

    if (!open) {
      return null;
    }

    return (
      <ModalRegister
        dappId={ dappId }
        onClose={ this.handleClose }
      />
    );
  }

  handleOpen = () => {
    const dappId = Math.random().toString();

    this.setState({ open: true, dappId });
  }

  handleClose = () => {
    this.setState({ open: false, dappId: null });
  }
}
