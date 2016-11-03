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
import { IconButton } from 'material-ui';
import Clipboard from 'react-copy-to-clipboard';
import CopyIcon from 'material-ui/svg-icons/content/content-copy';

export default class CopyToClipboard extends Component {
  static propTypes = {
    data: PropTypes.string.isRequired,
    label: PropTypes.string,
    onCopy: PropTypes.func,
    size: PropTypes.number, // in px
    cooldown: PropTypes.number // in ms
  };

  static defaultProps = {
    className: '',
    label: 'copy to clipboard',
    onCopy: () => {},
    size: 16,
    cooldown: 1000
  };

  state = {
    disabled: false
  };

  render () {
    const { data, label } = this.props;

    return (
      <Clipboard onCopy={ this.onCopy } text={ data }>
        <IconButton
          tooltip={ label }
        >
          <CopyIcon />
        </IconButton>
      </Clipboard>
    );
  }

  onCopy () {
    const { cooldown, onCopy } = this.props;

    this.setState({ disabled: true });
    setTimeout(() => {
      this.setState({ disabled: false });
    }, cooldown);

    onCopy();
  }
}
