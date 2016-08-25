import React, { Component, PropTypes } from 'react';
import blockies from 'blockies';

import styles from './style.css';

export default class IdentityIcon extends Component {
  static contextTypes = {
    contracts: React.PropTypes.array
  }

  static propTypes = {
    address: PropTypes.string,
    center: PropTypes.bool,
    padded: PropTypes.bool,
    inline: PropTypes.bool
  }

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
    const { inline } = this.props;
    const address = _address.toLowerCase();
    const contract = (this.context.contracts || []).find((c) => c.address.toLowerCase() === address);

    if (contract && contract.images) {
      this.setState({
        iconsrc: inline
          ? contract.images.small
          : contract.images.normal
      });

      return;
    }

    this.setState({
      iconsrc: blockies({
        seed: address,
        size: 8,
        scale: inline ? 4 : 7
      }).toDataURL()
    });
  }

  render () {
    const className = `${styles.icon} ${this.props.center ? styles.center : styles.right} ${this.props.padded ? styles.padded : null} ${this.props.inline ? styles.inline : null}`;

    return (
      <div className={ className }>
        <img
          src={ this.state.iconsrc }
          value={ this.props.address } />
      </div>
    );
  }
}
