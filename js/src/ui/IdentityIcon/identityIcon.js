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
