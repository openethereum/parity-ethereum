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

import Modal from '../Modal';
import Store from '../store';

export default class ModalDelete extends Component {
  static propTypes = {
    visible: PropTypes.bool.isRequired
  }

  store = Store.instance();

  render () {
    const { visible } = this.props;
    const buttons = [];

    return (
      <Modal
        buttons={ buttons }
        header='Confirm Application Deletion'
        visible={ visible }>
        This is just some info
      </Modal>
    );
  }
}
