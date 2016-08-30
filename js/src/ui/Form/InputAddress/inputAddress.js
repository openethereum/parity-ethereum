import React, { Component, PropTypes } from 'react';

import Input from '../Input';
import IdentityIcon from '../../IdentityIcon';

import styles from './style.css';

export default class InputAddress extends Component {
  static propTypes = {
    disabled: PropTypes.bool,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    onChange: PropTypes.func
  };

  render () {
    const { disabled, error, label, hint, value, onChange } = this.props;

    return (
      <div className={ styles.container }>
        <Input
          className={ styles.input }
          disabled={ disabled }
          label={ label }
          hint={ hint }
          error={ error }
          value={ value }
          onChange={ onChange } />
        { this.renderIcon() }
      </div>
    );
  }

  renderIcon () {
    const { value } = this.props;

    if (!value || !value.length) {
      return null;
    }

    return (
      <div className={ styles.icon }>
        <IdentityIcon
          inline center
          address={ value } />
      </div>
    );
  }
}
