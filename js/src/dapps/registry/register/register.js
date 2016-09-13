import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import RaisedButton from 'material-ui/RaisedButton';
import CheckIcon from 'material-ui/svg-icons/navigation/check';

import { fromWei } from '../parity.js';

import styles from './register.css';

export default class Register extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    names: PropTypes.array.isRequired,
    fee: PropTypes.object.isRequired
  }

  state = { name: '' };

  render () {
    const { name } = this.state;
    const { fee } = this.props;

    return (
      <Card className={ styles.register }>
        <CardHeader title={ 'Register a Name' } />
        <div className={ styles.box }>
          <TextField
            className={ styles.spacing }
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          <RaisedButton
            className={ styles.spacing }
            label='Register'
            primary
            icon={ <CheckIcon /> }
            onClick={ this.onRegisterClick }
          />
        </div>
        <CardText>
          <p>The registration fee is <code>{ fromWei(fee).toFixed(3) }</code>ΞTH.</p>
        </CardText>
      </Card>
    );
  }

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onRegisterClick = () => {
    this.props.actions.register(this.state.name);
  };
}
