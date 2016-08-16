import React, { Component, PropTypes } from 'react';

import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';

import styles from '../style.css';

export default class CreationType extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  componentWillMount () {
    this.props.onChange('fromNew');
  }

  render () {
    return (
      <div className={ styles.paddedtop }>
        <RadioButtonGroup
          defaultSelected='fromNew'
          name='creationType'
          onChange={ this.onChange }>
          <RadioButton
            label='Create new account via username & password'
            value='fromNew' />
          <RadioButton
            label='Import account from a backup JSON file'
            value='fromJSON' />
          <RadioButton
            label='Import account from an Ethereum pre-sale wallet'
            value='fromPresale' />
        </RadioButtonGroup>
      </div>
    );
  }

  onChange = (event) => {
    this.props.onChange(event.target.value);
  }
}
