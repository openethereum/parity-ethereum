import React, { Component, PropTypes } from 'react';

import { Chip } from 'material-ui';

import IdentityIcon from '../IdentityIcon' ;

import styles from './chip.css';

export default class CustomChip extends Component {
  static propTypes = {
    isAddress: PropTypes.bool,
    value: PropTypes.string,
    label: PropTypes.string,
    displayValue: PropTypes.string
  };

  render () {
    const { isAddress, value, label } = this.props;

    const displayValue = this.props.displayValue || value;

    return (
      <Chip
        className={ styles.chip }
        style={ {
          margin: '0.5em',
          background: 'rgb(50, 100, 150)',
          display: 'flex',
          flexDirection: 'column'
        } }>
        { this.renderIcon(isAddress, value) }
        <span className={ styles.value } title={ value }>
          { displayValue }
        </span>
        <span className={ styles.label }>
          { label }
        </span>
      </Chip>
    );
  }

  renderIcon (isAddress, address) {
    if (!isAddress) return;

    return (
      <IdentityIcon
        inline center
        address={ address } />
    );
  }
}
