import React, { Component, PropTypes } from 'react';
import blockies from 'blockies';

import styles from './style.css';

export default class IdentityIcon extends Component {
  static propTypes = {
    address: PropTypes.string,
    center: PropTypes.bool
  }

  state = {
    iconsrc: ''
  }

  componentDidMount () {
    this.updateIcon(this.props.address);
  }

  updateIcon (address) {
    this.setState({
      iconsrc: blockies({
        seed: address,
        size: 8,
        scale: 8
      }).toDataURL()
    });
  }

  render () {
    const className = `${styles.icon} ${this.props.center ? styles.center : styles.right}`;

    return (
      <div className={ className }>
        <img
          src={ this.state.iconsrc }
          value={ this.props.address } />
      </div>
    );
  }
}
