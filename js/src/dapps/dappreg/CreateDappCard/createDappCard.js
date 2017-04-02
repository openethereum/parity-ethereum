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

import DappsStore from '../dappsStore';
import Card from '../Card';
import ModalRegister from '../ModalRegister';

import PlusImage from '~/../assets/images/dapps/plus.svg';

export default class CreateDappCard extends Component {
  state = {
    dappId: null,
    focus: false,
    open: false
  };

  dappsStore = DappsStore.get();

  render () {
    const { focus } = this.state;

    return (
      <div>
        { this.renderModal() }

        <Card
          dashed
          focus={ focus }
          icon={ (<img src={ PlusImage } />) }
          name={ { value: 'Register a dapp' } }
          onClick={ this.handleOpen }
        />
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
        onRegister={ this.handleRegister }
      />
    );
  }

  handleOpen = () => {
    const dappId = this.dappsStore.createDappId();

    this.setState({ focus: false, open: true, dappId });
  }

  handleClose = () => {
    this.setState({ focus: true, open: false, dappId: null });
  }

  handleRegister = () => {
    const { dappId } = this.state;

    this.dappsStore.register(dappId);
    this.handleClose();
  }
}
