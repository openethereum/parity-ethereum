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
    hasAccount: PropTypes.bool.isRequired,
    pending: PropTypes.bool.isRequired,
    posted: PropTypes.array.isRequired
  }

  state = { name: '' };

  render () {
    const { name } = this.state;
    const { fee, hasAccount, pending, posted } = this.props;

    return (
      <Card className={ styles.register }>
        <CardHeader title={ 'Register a Name' } />
        <CardText>
          { !hasAccount
            ? (<p className={ styles.noSpacing }>Please select an account first.</p>)
            : (<p className={ styles.noSpacing }>The registration fee is <code>{ fromWei(fee).toFixed(3) }</code>ΞTH.</p>)
          }
          <TextField
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          <RaisedButton
            disabled={ !hasAccount || pending }
            className={ styles.spacing }
            label='Register'
            primary
            icon={ <CheckIcon /> }
            onClick={ this.onRegisterClick }
          />
          { posted.map((name) => (
            <p key={ name }>
              Please use the <a href='/#/signer' className={ styles.link } target='_blank'>Signer</a> to authenticate the registraction of <code>{ name }</code>.
            </p>
          )) }
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
