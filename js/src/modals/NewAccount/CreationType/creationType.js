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
      <div>
        <div className={ styles.info }>
          You can create an account either with a password or though an import from a pre-existing resource or export from another system
        </div>
        <RadioButtonGroup
          defaultSelected='fromNew'
          name='creationType'
          onChange={ this.onChange }>
          <RadioButton
            label='Create new account'
            value='fromNew' />
          <RadioButton
            label='Import account from JSON file'
            value='fromJSON' />
          <RadioButton
            label='Import account from pre-sale wallet'
            value='fromPresale' />
        </RadioButtonGroup>
      </div>
    );
  }

  onChange = (event) => {
    this.props.onChange(event.target.value);
  }
}
