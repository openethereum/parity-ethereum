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
import Theme from '../Theme';
const { textColor, disabledTextColor } = Theme.flatButton;

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
    copied: false
  };

  render () {
    const { data, label, size } = this.props;
    const { copied } = this.state;

    return (
      <Clipboard onCopy={ this.onCopy } text={ data }>
        <IconButton
          tooltip={ copied ? 'done!' : label }
          disableTouchRipple
          tooltipPosition={ 'top-right' }
          tooltipStyles={ { marginTop: `-${size / 4}px` } }
          style={ { width: size, height: size, padding: '0' } }
          iconStyle={ { width: size, height: size } }
        >
          <CopyIcon color={ copied ? disabledTextColor : textColor } />
        </IconButton>
      </Clipboard>
    );
  }

  onCopy = () => {
    const { cooldown, onCopy } = this.props;

    this.setState({ copied: true });
    setTimeout(() => {
      this.setState({ copied: false });
    }, cooldown);

    onCopy();
  }
}
