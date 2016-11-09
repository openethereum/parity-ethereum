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

import ContentClear from 'material-ui/svg-icons/content/clear';

import { Button, Modal, Editor } from '../../ui';

export default class LoadContract extends Component {

  static propTypes = {
    onClose: PropTypes.func.isRequired,
    onLoad: PropTypes.func.isRequired,
    contracts: PropTypes.object.isRequired
  };

  state = {
  };

  render () {
    return (
      <Modal
        title='load contract'
        actions={ this.renderDialogActions() }
        visible
      >
        <div>
          <p>Choose a contract to load</p>
          { this.renderContracts() }
        </div>
      </Modal>
    );
  }

  renderContracts () {
    const { contracts } = this.props;

    return Object
      .values(contracts)
      .map((contract) => {
        const { id, name, timestamp, sourcecode } = contract;

        return (
          <div key={ id }>
            { name }
          </div>
        );
      });
  }

  renderDialogActions () {
    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose }
      />
    );

    return [ cancelBtn ];
  }

  onClose = () => {
    this.props.onClose();
  }

}
