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
import PropTypes from 'prop-types';

import { createIdentityImg } from '@parity/api/util/identity';
import { isNullAddress } from '@parity/shared/util/validation';

import IconCache from '../IconCache';
import { CancelIcon, ContractIcon } from '../Icons';

import styles from './identityIcon.css';

const iconCache = IconCache.get();

export default class IdentityIcon extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string,
    button: PropTypes.bool,
    center: PropTypes.bool,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    inline: PropTypes.bool,
    padded: PropTypes.bool,
    tiny: PropTypes.bool
  }

  static iconCache = iconCache;

  state = {
    iconsrc: ''
  }

  componentDidMount () {
    this.updateIcon(this.props.address);
  }

  componentWillReceiveProps (newProps) {
    if (newProps.address === this.props.address) {
      return;
    }

    this.updateIcon(newProps.address);
  }

  updateIcon (_address) {
    const { api } = this.context;
    const { button, inline, tiny } = this.props;

    if (iconCache.images[_address]) {
      this.setState({ iconsrc: `${api.dappsUrl}${iconCache.images[_address]}` });
      return;
    }

    let scale = 7;

    if (tiny) {
      scale = 2;
    } else if (button) {
      scale = 3;
    } else if (inline) {
      scale = 4;
    }

    this.setState({
      iconsrc: createIdentityImg(_address, scale)
    });
  }

  render () {
    const { address, button, className, center, disabled, inline, padded, tiny } = this.props;
    const { iconsrc } = this.state;
    const classes = [
      styles.icon,
      disabled ? styles.disabled : '',
      tiny ? styles.tiny : '',
      button ? styles.button : '',
      center ? styles.center : styles.left,
      inline ? styles.inline : '',
      padded ? styles.padded : '',
      className
    ].join(' ');

    let size = '56px';

    if (tiny) {
      size = '16px';
    } else if (button) {
      size = '24px';
    } else if (inline) {
      size = '32px';
    }

    if (!address) {
      return (
        <ContractIcon
          className={ classes }
          data-address-img
          style={ {
            background: '#eee',
            height: size,
            width: size
          } }
        />
      );
    } else if (isNullAddress(address)) {
      return (
        <CancelIcon
          className={ classes }
          data-address-img
          style={ {
            background: '#333',
            height: size,
            width: size
          } }
        />
      );
    }

    return (
      <img
        className={ classes }
        data-address-img
        height={ size }
        width={ size }
        src={ iconsrc }
      />
    );
  }
}
