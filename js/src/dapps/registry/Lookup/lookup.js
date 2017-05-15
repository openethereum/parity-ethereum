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

import { observer } from 'mobx-react';
import React, { Component } from 'react';
import { Card, CardText } from 'material-ui/Card';
import CircularProgress from 'material-ui/CircularProgress';

import ApplicationStore from '../Application/application.store';
import LookupStore from './lookup.store';

import Entry from '../Entry';

import styles from './lookup.css';

@observer
export default class Lookup extends Component {
  applicationStore = ApplicationStore.get();
  lookupStore = LookupStore.get();

  render () {
    const { inputValue } = this.lookupStore;

    return (
      <div>
        <div className={ styles.inputContainer }>
          <input
            autoFocus
            className={ styles.input }
            placeholder='Type a name'
            onChange={ this.handleInputChange }
            value={ inputValue }
          />
        </div>
        { this.renderOutput() }
      </div>
    );
  }

  renderReserving (name) {
    return (
      <Card className={ styles.container }>
        <CardText>
          <div className={ styles.reserving }>
            <CircularProgress size={ 25 } />
            <div>
              Reserving <code>{ name }</code>...
            </div>
          </div>
        </CardText>
      </Card>
    );
  }

  renderOutput () {
    const { inputValue, result, reserving } = this.lookupStore;

    if (reserving) {
      return this.renderReserving(reserving);
    }

    if (!result || !inputValue) {
      return null;
    }

    if (result.free) {
      return this.renderFreeName(result.name);
    }

    return (
      <Entry entry={ result } />
    );
  }

  renderFreeName (name) {
    const { api, fee } = this.applicationStore;

    return (
      <Card className={ styles.container }>
        <CardText>
          <div
            className={ styles.free }
            onClick={ this.handleRegister }
          >
            This name has not been reserved yet.
            Click to
            reserve <code>{ name }</code> for { api.util.fromWei(fee).toFormat(3) } ETH
          </div>
        </CardText>
      </Card>
    );
  }

  handleInputChange = (e) => {
    const { value } = e.target;

    return this.lookupStore.updateInput(value);
  };

  handleRegister = () => {
    this.lookupStore.register();
  };
}
