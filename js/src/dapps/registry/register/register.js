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
    fee: PropTypes.object.isRequired,
    pending: PropTypes.bool.isRequired,
    posted: PropTypes.array.isRequired
  }

  state = { name: '' };

  render () {
    const { name } = this.state;
    const { fee, pending, posted } = this.props;

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
            disabled={ pending }
            className={ styles.spacing }
            label='Register'
            primary
            icon={ <CheckIcon /> }
            onClick={ this.onRegisterClick }
          />
        </div>
        <CardText>
          { posted.map((name) => (
            <p key={ name }>
              Please use the <a href='/#/signer' target='_blank'>Signer</a> to authenticate
              the registraction of <code>{ name }</code>.
            </p>
          )) }
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
