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

import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Modal } from '../../ui';

const STAGE_NAMES = ['fund account'];

export default class FundAccount extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    onClose: PropTypes.func
  }

  state = {
    stage: 0
  }

  render () {
    const { stage } = this.state;

    return (
      <Modal
        actions={ this.renderDialogActions() }
        current={ stage }
        steps={ STAGE_NAMES }
        visible>
        <div>
          Placeholder until such time as we have the ShapeShift.io integration going (just time, a scarce commodity)
        </div>
      </Modal>
    );
  }

  renderDialogActions () {
    return (
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />
    );
  }

  onClose = () => {
    this.setState({
      stage: 0
    }, () => {
      this.props.onClose && this.props.onClose();
    });
  }
}
