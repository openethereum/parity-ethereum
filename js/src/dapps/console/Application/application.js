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

import React, { Component } from 'react';

import { api } from '../parity';
import ConsoleStore from '../consoleStore';
import Eval from '../Eval';
import Input from '../Input';
import Status from '../Status';

import styles from './application.css';

export default class Application extends Component {
  consoleStore = ConsoleStore.get();

  componentWillMount () {
    this.consoleStore.addWatch('time', () => new Date());
    this.consoleStore.addWatch('blockNumber', api.eth.blockNumber, api);
  }

  render () {
    return (
      <div className={ styles.app }>
        <div className={ styles.eval }>
          <Eval />
        </div>
        <div className={ styles.input }>
          <Input />
        </div>
        <div className={ styles.status }>
          <Status />
        </div>
      </div>
    );
  }
}
