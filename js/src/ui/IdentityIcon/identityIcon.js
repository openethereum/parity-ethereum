import React, { Component, PropTypes } from 'react';

import styles from './identityIcon.css';

export default class IdentityIcon extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

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
    const { address } = this.props;

    this.updateIcon(address);
  }

  componentWillReceiveProps (newProps) {
    const { address, tokens } = this.props;

    if (newProps.address === address && newProps.tokens === tokens) {
      return;
    }

    this.updateIcon(newProps.address);
  }

  updateIcon (_address) {
    const { api } = this.context;
    const { tokens, inline } = this.props;
    const token = (tokens || {})[_address];

    if (token && token.images) {
      this.setState({
        iconsrc: token.images[inline ? 'small' : 'normal']
      });

      return;
    }

    this.setState({
      iconsrc: api.util.createIdentityImg(_address, inline ? 4 : 7)
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
