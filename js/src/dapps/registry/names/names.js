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
import DropDownMenu from 'material-ui/DropDownMenu';
import MenuItem from 'material-ui/MenuItem';
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
    reserved: PropTypes.array.isRequired,
    dropped: PropTypes.array.isRequired
  }

  state = {
    action: 'reserve',
    name: ''
  };

  render () {
    const { action, name } = this.state;
    const { fee, hasAccount, pending, reserved, dropped } = this.props;

    return (
      <Card className={ styles.names }>
        <CardHeader title={ 'Reserve Names' } />
        <CardText>
          { !hasAccount
            ? (<p className={ styles.noSpacing }>Please select an account first.</p>)
            : (<p className={ styles.noSpacing }>
                The fee to reserve a name is <code>{ fromWei(fee).toFixed(3) }</code>ΞTH.
                To drop a name, you have to be the owner.
              </p>)
          }
          <TextField
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          <DropDownMenu
            disabled={ !hasAccount || pending }
            value={ action }
            onChange={ this.onActionChange }
          >
            <MenuItem value='reserve' primaryText='reserve this name' />
            <MenuItem value='drop' primaryText='drop this name' />
          </DropDownMenu>
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
              Please use the <a href='/#/signer' className={ styles.link } target='_blank'>Signer</a> to authenticate reserving <code>{ name }</code>.
            </p>
          )) }
          { dropped.map((name) => (
            <p key={ name }>
              Please use the <a href='/#/signer' className={ styles.link } target='_blank'>Signer</a> to authenticate dropping <code>{ name }</code>.
            </p>
          )) }
        </CardText>
      </Card>
    );
  }

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onActionChange = (e, i, action) => {
    this.setState({ action });
  };
  onSubmitClick = () => {
    const { action, name } = this.state;
    if (action === 'reserve') {
      this.props.actions.reserve(name);
    } else if (action === 'drop') {
      this.props.actions.drop(name);
    }
  };
}
