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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ContractIcon from 'material-ui/svg-icons/action/code';

import { memorizeIcon } from '../../redux/providers/imagesActions';

import styles from './identityIcon.css';

class IdentityIcon extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    memorizeIcon: PropTypes.func.isRequired,

    address: PropTypes.string,
    button: PropTypes.bool,
    className: PropTypes.string,
    center: PropTypes.bool,
    padded: PropTypes.bool,
    inline: PropTypes.bool,
    tiny: PropTypes.bool,
    image: PropTypes.string,
    icon: PropTypes.object,
    memorize: PropTypes.bool
  };

  static defaultProps = {
    memorize: false
  };

  state = {
    iconsrc: ''
  }

  shouldComponentUpdate (newProps, newState) {
    return newProps.address !== this.props.address;
  }

  componentWillMount () {
    const scale = this.getScale();
    const iconsrc = this.getIconSrc(scale);

    this.setState({ iconsrc });
  }

  getScale () {
    const { button, inline, tiny } = this.props;

    if (tiny) return 2;
    if (button) return 3;
    if (inline) return 4;

    return 7;
  }

  getIconSrc (scale) {
    const { api } = this.context;
    const { address, image, icon, memorize } = this.props;

    if (!address) {
      return;
    }

    if (image) {
      return `${api.dappsUrl}${image}`;
    }

    if (icon && icon[scale]) {
      return icon[scale];
    }

    const iconsrc = api.util.createIdentityImg(address, scale);

    if (memorize && iconsrc) {
      this.props.memorizeIcon(address, scale, iconsrc);
    }

    return iconsrc;
  }

  render () {
    const { button, className, center, inline, padded, tiny } = this.props;
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

    if (!iconsrc) {
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

function mapStateToProps (_, initProps) {
  const { address } = initProps;

  return (state) => {
    const { images } = state;
    const { icons } = images;

    const image = images[address];
    const icon = icons[address] || {};

    return { image, icon };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    memorizeIcon
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(IdentityIcon);
