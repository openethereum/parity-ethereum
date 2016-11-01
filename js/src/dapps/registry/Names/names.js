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

const useSignerText = (<p>Use the <a href='/#/signer' className={ styles.link } target='_blank'>Signer</a> to authenticate the following changes.</p>);

const renderNames = (names) => {
  const values = Object.values(names);

  return values
    .map((name, index) => (
      <span key={ index }>
        <code>{ name }</code>
        {
          index < values.length - 1
          ? (<span>, </span>)
          : null
        }
      </span>
    ));
};

const renderQueue = (queue) => {
  if (queue.length === 0) {
    return null;
  }

  const grouped = queue.reduce((grouped, change) => {
    const last = grouped[grouped.length - 1];
    if (last && last.action === change.action) {
      last.names.push(change.name);
    } else {
      grouped.push({ action: change.action, names: [change.name] });
    }
    return grouped;
  }, []);

  return (
    <ul>
      { grouped.map(({ action, names }) => (
        <li key={ action + '-' + names.join('-') }>
          { renderNames(names) }
          { ' will be ' }
          { action === 'reserve' ? 'reserved' : 'dropped' }
        </li>
      )) }
    </ul>
  );
};

export default class Names extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    fee: PropTypes.object.isRequired,
    pending: PropTypes.bool.isRequired,
    queue: PropTypes.array.isRequired
  }

  state = {
    action: 'reserve',
    name: ''
  };

  render () {
    const { action, name } = this.state;
    const { fee, pending, queue } = this.props;

    return (
      <Card className={ styles.names }>
        <CardHeader title={ 'Manage Names' } />
        <CardText>
          { (action === 'reserve'
              ? (<p className={ styles.noSpacing }>
                The fee to reserve a name is <code>{ fromWei(fee).toFixed(3) }</code>ETH.
              </p>)
              : (<p className={ styles.noSpacing }>To drop a name, you have to be the owner.</p>)
            )
          }
          <TextField
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          <DropDownMenu
            disabled={ pending }
            value={ action }
            onChange={ this.onActionChange }
          >
            <MenuItem value='reserve' primaryText='reserve this name' />
            <MenuItem value='drop' primaryText='drop this name' />
          </DropDownMenu>
          <RaisedButton
            disabled={ pending }
            className={ styles.spacing }
            label={ action === 'reserve' ? 'Reserve' : 'Drop' }
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
  onSubmitClick = () => {
    const { action, name } = this.state;
    if (action === 'reserve') {
      this.props.actions.reserve(name);
    } else if (action === 'drop') {
      this.props.actions.drop(name);
    }
  };
}
