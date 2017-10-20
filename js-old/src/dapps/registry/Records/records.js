// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Card, CardHeader, CardText } from 'material-ui/Card';
import TextField from 'material-ui/TextField';
import DropDownMenu from 'material-ui/DropDownMenu';
import MenuItem from 'material-ui/MenuItem';
import RaisedButton from 'material-ui/RaisedButton';
import SaveIcon from 'material-ui/svg-icons/content/save';

import { nullableProptype } from '~/util/proptypes';
import { clearError, update } from './actions';
import styles from './records.css';

class Records extends Component {
  static propTypes = {
    error: nullableProptype(PropTypes.object.isRequired),
    pending: PropTypes.bool.isRequired,
    name: PropTypes.string.isRequired,
    type: PropTypes.string.isRequired,
    value: PropTypes.string.isRequired,

    clearError: PropTypes.func.isRequired,
    update: PropTypes.func.isRequired
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
          { this.renderError() }
          <div className={ styles.box }>
            <TextField
              hintText='name'
              value={ name }
              onChange={ this.onNameChange }
            />
            <DropDownMenu
              value={ type }
              onChange={ this.onTypeChange }
            >
              <MenuItem value='A' primaryText='A – Ethereum address' />
              <MenuItem value='IMG' primaryText='IMG – hash of a picture in the blockchain' />
              <MenuItem value='CONTENT' primaryText='CONTENT – hash of a data in the blockchain' />
            </DropDownMenu>
            <TextField
              hintText='value'
              value={ value }
              onChange={ this.onValueChange }
            />
            <div className={ styles.button }>
              <RaisedButton
                disabled={ pending }
                className={ styles.spacing }
                label='Save'
                primary
                icon={ <SaveIcon /> }
                onTouchTap={ this.onSaveClick }
              />
            </div>
          </div>
        </CardText>
      </Card>
    );
  }

  renderError () {
    const { error } = this.props;

    if (!error) {
      return null;
    }

    return (
      <div className={ styles.error }>
        <code>{ error.message }</code>
      </div>
    );
  }

  onNameChange = (e) => {
    this.clearError();
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

    this.props.update(name, type, value);
  };

  clearError = () => {
    if (this.props.error) {
      this.props.clearError();
    }
  };
}

const mapStateToProps = (state) => state.records;
const mapDispatchToProps = (dispatch) => bindActionCreators({ clearError, update }, dispatch);

export default connect(mapStateToProps, mapDispatchToProps)(Records);
