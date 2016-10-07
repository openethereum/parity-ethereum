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
import ContractIcon from 'material-ui/svg-icons/action/code';

import styles from './identityIcon.css';

export default class IdentityIcon extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string,
    button: PropTypes.bool,
    className: PropTypes.string,
    center: PropTypes.bool,
    padded: PropTypes.bool,
    inline: PropTypes.bool,
    tiny: PropTypes.bool
  }

  state = {
    iconsrc: ''
  }

  componentDidMount () {
    const { address } = this.props;

    this.updateIcon(address);
  }

  componentWillReceiveProps (newProps) {
    const { address } = this.props;

    if (newProps.address === address) {
      return;
    }

    this.updateIcon(newProps.address);
  }

  updateIcon (_address) {
    const { api } = this.context;
    const { button, inline, tiny } = this.props;
    // const token = (tokens || {})[_address];
    //
    // if (token && token.image) {
    //   this.setState({
    //     iconsrc: token.image
    //   });
    //
    //   return;
    // }

    let scale = 7;
    if (tiny) {
      scale = 2;
    } else if (button) {
      scale = 3;
    } else if (inline) {
      scale = 4;
    }

    this.setState({
      iconsrc: api.util.createIdentityImg(_address, scale)
    });
  }

  render () {
    const { address, button, className, center, inline, padded, tiny } = this.props;
    const { iconsrc } = this.state;
    const classes = [
      styles.icon,
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
          style={ { width: size, height: size, background: '#eee' } } />
      );
    }

    return (
      <img
        className={ classes }
        src={ iconsrc }
        width={ size }
        height={ size } />
    );
  }
}
