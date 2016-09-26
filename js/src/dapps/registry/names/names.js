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

const useSignerText = (<p>Use the <a href='/#/signer' className={ styles.link } target='_blank'>Signer</a> to authenticate the following changes.</p>)

const renderNames = (names) => {
  const out = []
  for (let name of names) {
    out.push((<code>{ name }</code>), ', ')
  }
  out.pop()
  return out
}

const renderQueue = (queue) => {
  if (queue.length === 0) {
    return null;
  }

  const grouped = queue.reduce((grouped, change) => {
    const last = grouped[grouped.length - 1]
    if (last && last.action === change.action) {
      last.names.push(change.name)
    } else {
      grouped.push({action: change.action, names: [change.name]})
    }
    return grouped
  }, []);

  return (
    <ul>
      { grouped.map(({action, names}) => (
        <li key={ action + '-' + names.join('-') }>
          { <code>{ action }</code> }
          { ' ' }
          { renderNames(names) }
        </li>
      )) }
    </ul>
  );
}

export default class Names extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    fee: PropTypes.object.isRequired,
    hasAccount: PropTypes.bool.isRequired,
    pending: PropTypes.bool.isRequired,
    queue: PropTypes.array.isRequired
  }

  state = {
    action: 'reserve',
    name: '',
    receiver: ''
  };

  render () {
    const { action, name, receiver } = this.state;
    const { fee, hasAccount, pending, queue } = this.props;

    const notes = {
      reserve: (
        <p className={ styles.noSpacing }>
          The fee to reserve a name is <code>{ fromWei(fee).toFixed(3) }</code>ΞTH.
        </p>
      ),
      drop: (
        <p className={ styles.noSpacing }>To drop a name, you have to be the owner.</p>
      ),
      transfer: (
        <p className={ styles.noSpacing }>
          To transfer a name, you have to be the owner.
          { ' ' }
          <strong>If the new owner is not a valid address, the name will be lost!</strong>
        </p>
      )
    }[action] || null;

    return (
      <Card className={ styles.names }>
        <CardHeader title={ 'Manage Names' } />
        <CardText>
          { !hasAccount
            ? (<p className={ styles.noSpacing }>Please select an account first.</p>)
            : notes
          }
          <TextField
            disabled={ !hasAccount || pending }
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
            <MenuItem value='transfer' primaryText='transfer this name' />
          </DropDownMenu>
          <TextField
            disabled={ !hasAccount || pending }
            style={ action === 'transfer' ? {} : { display: 'none' } }
            hintText='new owner'
            value={ receiver }
            onChange={ this.onReceiverChange }
          />
          <RaisedButton
            disabled={ !hasAccount || pending }
            className={ styles.spacing }
            label={ action }
            primary
            icon={ <CheckIcon /> }
            onClick={ this.onSubmitClick }
          />
          { queue.length > 0
            ? (<div>{ useSignerText }{ renderQueue(queue) }</div>)
            : null
          }
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
  onReceiverChange = (e) => {
    this.setState({ receiver: e.target.value });
  };
  onSubmitClick = () => {
    const { action, name, receiver } = this.state;
    if (action === 'reserve') {
      this.props.actions.reserve(name);
      this.setState({ name: '' });
    } else if (action === 'drop') {
      this.props.actions.drop(name);
      this.setState({ name: '' });
    } else if (action === 'transfer') {
      this.props.actions.transfer(name, receiver);
      this.setState({ name: '', receiver: '' });
    }
  };
}
