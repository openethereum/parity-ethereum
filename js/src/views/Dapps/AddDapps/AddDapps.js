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
import Dialog from 'material-ui/Dialog';
import IconButton from 'material-ui/IconButton';
import DoneIcon from 'material-ui/svg-icons/action/done';

export default class AddDapps extends Component {
  static propTypes = {
    open: PropTypes.bool.isRequired,
    onAdd: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired
  };

  render () {
    const actions = [
      <IconButton
        label={ 'Done' }
        onClick={ this.props.onClose }
      >
        <DoneIcon />
      </IconButton>
    ]

    return (
      <Dialog
        visible
        title='Select Distributed Apps to be shown'
        actions={ actions }
        open={ this.props.open }>
        foo bar baz
      </Dialog>
    );
  }
}
