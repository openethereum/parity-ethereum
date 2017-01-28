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
import {
  Card, CardHeader, CardText, TextField, DropDownMenu, MenuItem, RaisedButton
} from 'material-ui';

import { nullableProptype } from '~/util/proptypes';
import { AddIcon, CheckIcon } from '~/ui/Icons';
import { clearError, confirm, propose } from './actions';
import styles from './reverse.css';

class Reverse extends Component {
  static propTypes = {
    error: nullableProptype(PropTypes.object.isRequired),
    pending: PropTypes.bool.isRequired,
    queue: PropTypes.array.isRequired,

    clearError: PropTypes.func.isRequired,
    confirm: PropTypes.func.isRequired,
    propose: PropTypes.func.isRequired
  }

  state = {
    action: 'propose',
    name: '',
    address: ''
  };

  render () {
    const { pending } = this.props;
    const { action, address, name } = this.state;

    const explanation = action === 'propose'
      ? (
        <p className={ styles.noSpacing }>
          To propose a reverse entry for <code>foo</code>, you have to be the owner of it.
        </p>
      ) : (
        <p className={ styles.noSpacing }>
          To confirm a proposal, send the transaction from the account that the name has been proposed for.
        </p>
      );

    let addressInput = null;

    if (action === 'propose') {
      addressInput = (
        <TextField
          className={ styles.spacing }
          hintText='address'
          value={ address }
          onChange={ this.onAddressChange }
        />
      );
    }

    return (
      <Card className={ styles.reverse }>
        <CardHeader title={ 'Manage Reverse Names' } />
        <CardText>
          <p className={ styles.noSpacing }>
            <strong>
              To make others to find the name of an address using the registry, you can propose & confirm reverse entries.
            </strong>
          </p>
          { explanation }
          { this.renderError() }
          <div className={ styles.box }>
            <DropDownMenu
              disabled={ pending }
              value={ action }
              onChange={ this.onActionChange }
            >
              <MenuItem value='propose' primaryText='propose a reverse entry' />
              <MenuItem value='confirm' primaryText='confirm a reverse entry' />
            </DropDownMenu>
            { addressInput }
            <TextField
              className={ styles.spacing }
              hintText='name'
              value={ name }
              onChange={ this.onNameChange }
            />
            <div className={ styles.button }>
              <RaisedButton
                disabled={ pending }
                label={ action === 'propose' ? 'Propose' : 'Confirm' }
                primary
                icon={ action === 'propose' ? <AddIcon /> : <CheckIcon /> }
                onTouchTap={ this.onSubmitClick }
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
    this.setState({ name: e.target.value });
  };

  onAddressChange = (e) => {
    this.setState({ address: e.target.value });
  };

  onActionChange = (e, i, action) => {
    this.setState({ action });
  };

  onSubmitClick = () => {
    const { action, name, address } = this.state;

    if (action === 'propose') {
      this.props.propose(name, address);
    } else if (action === 'confirm') {
      this.props.confirm(name);
    }
  };

  clearError = () => {
    if (this.props.error) {
      this.props.clearError();
    }
  };
}

const mapStateToProps = (state) => state.reverse;
const mapDispatchToProps = (dispatch) => bindActionCreators({ clearError, confirm, propose }, dispatch);

export default connect(mapStateToProps, mapDispatchToProps)(Reverse);
