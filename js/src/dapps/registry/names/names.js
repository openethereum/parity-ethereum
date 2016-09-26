// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import RaisedButton from 'material-ui/RaisedButton';
import CheckIcon from 'material-ui/svg-icons/navigation/check';

import { fromWei } from '../parity.js';

import styles from './names.css';

export default class Names extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    fee: PropTypes.object.isRequired,
    hasAccount: PropTypes.bool.isRequired,
    pending: PropTypes.bool.isRequired,
    reserved: PropTypes.array.isRequired
  }

  state = { name: '' };

  render () {
    const { name } = this.state;
    const { fee, hasAccount, pending, reserved } = this.props;

    return (
      <Card className={ styles.names }>
        <CardHeader title={ 'Reserve Names' } />
        <CardText>
          { !hasAccount
            ? (<p className={ styles.noSpacing }>Please select an account first.</p>)
            : (<p className={ styles.noSpacing }>The fee to reserve a name is <code>{ fromWei(fee).toFixed(3) }</code>ΞTH.</p>)
          }
          <TextField
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          <RaisedButton
            disabled={ !hasAccount || pending }
            className={ styles.spacing }
            label='Reserve'
            primary
            icon={ <CheckIcon /> }
            onClick={ this.onSubmitClick }
          />
          { reserved.map((name) => (
            <p key={ name }>
              Please use the <a href='/#/signer' className={ styles.link } target='_blank'>Signer</a> to authenticate the registration of <code>{ name }</code>.
            </p>
          )) }
        </CardText>
      </Card>
    );
  }

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onSubmitClick = () => {
    this.props.actions.reserve(this.state.name);
  };
}
