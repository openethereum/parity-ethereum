// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import SaveIcon from 'material-ui/svg-icons/content/save';

import recordTypeSelect from '../ui/record-type-select.js';
import styles from './records.css';

export default class Records extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    pending: PropTypes.bool.isRequired,
    name: PropTypes.string.isRequired,
    type: PropTypes.string.isRequired,
    value: PropTypes.string.isRequired
  }

  state = { name: '', type: 'A', value: '' };

  render () {
    const { pending } = this.props;
    const name = this.state.name || this.props.name;
    const type = this.state.type || this.props.type;
    const value = this.state.value || this.props.value;

    return (
      <Card className={ styles.records }>
        <CardHeader title={ 'Manage Entries of a Name' } />
        <CardText>
          <p className={ styles.noSpacing }>
            You can only modify entries of names that you previously registered.
          </p>

          <TextField
            className={ styles.spacing }
            hintText='name'
            value={ name }
            onChange={ this.onNameChange }
          />
          { recordTypeSelect(type, this.onTypeChange, styles.spacing) }
          <TextField
            className={ styles.spacing }
            hintText='value'
            value={ value }
            onChange={ this.onValueChange }
          />
          <RaisedButton
            disabled={ pending }
            className={ styles.spacing }
            label='Save'
            primary
            icon={ <SaveIcon /> }
            onTouchTap={ this.onSaveClick }
          />
        </CardText>
      </Card>
    );
  }

  onNameChange = (e) => {
    this.setState({ name: e.target.value });
  };
  onTypeChange = (e, i, type) => {
    this.setState({ type });
  };
  onValueChange = (e) => {
    this.setState({ value: e.target.value });
  };
  onSaveClick = () => {
    const { name, type, value } = this.state;
    this.props.actions.update(name, type, value);
  };
}
