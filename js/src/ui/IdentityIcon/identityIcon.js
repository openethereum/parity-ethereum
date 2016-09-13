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
import blockies from 'blockies';

import styles from './identityIcon.css';

export default class IdentityIcon extends Component {
  static propTypes = {
    address: PropTypes.string,
    className: PropTypes.string,
    center: PropTypes.bool,
    padded: PropTypes.bool,
    inline: PropTypes.bool,
    tokens: PropTypes.object
  }

  state = {
    iconsrc: ''
  }

  componentDidMount () {
    this.updateIcon(this.props.address);
  }

  componentWillReceiveProps (newProps) {
    const { address, tokens } = this.props;

    if (newProps.address === address && newProps.tokens === tokens) {
      return;
    }

    this.updateIcon(newProps.address);
  }

  updateIcon (_address) {
    const { tokens, inline } = this.props;
    const token = (tokens || {})[_address];

    if (token && token.images) {
      this.setState({
        iconsrc: token.images[inline ? 'small' : 'normal']
      });

      return;
    }

    const address = _address.toLowerCase();

    this.setState({
      iconsrc: blockies({
        seed: address,
        size: 8,
        scale: inline ? 4 : 7
      }).toDataURL()
    });
  }

  render () {
    const { className, center, inline, padded } = this.props;
    const { iconsrc } = this.state;
    const size = inline ? '32px' : '56px';
    const classes = `${styles.icon} ${center ? styles.center : styles.left} ${padded ? styles.padded : ''} ${inline ? styles.inline : ''} ${className}`;

    return (
      <img
        className={ classes }
        src={ iconsrc }
        width={ size }
        height={ size } />
    );
  }
}
