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
import { observer } from 'mobx-react';

import { Actionbar, Page } from '../../ui';
import FlatButton from 'material-ui/FlatButton';
import EyeIcon from 'material-ui/svg-icons/image/remove-red-eye';

import DappsStore from './dappsStore';

import AddDapps from './AddDapps';
import Summary from './Summary';

import styles from './dapps.css';

@observer
export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  store = new DappsStore(this.context.api);

  render () {
    return (
      <div>
        <AddDapps store={ this.store } />
        <Actionbar
          className={ styles.toolbar }
          title='Decentralized Applications'
          buttons={ [
            <FlatButton
              label='edit'
              key='edit'
              icon={ <EyeIcon /> }
              onClick={ this.store.openModal }
            />
          ] }
        />
        <Page>
          <div className={ styles.list }>
            { this.store.visible.map(this.renderApp) }
          </div>
        </Page>
      </div>
    );
  }

  renderApp = (app) => {
    return (
      <div
        className={ styles.item }
        key={ app.id }>
        <Summary app={ app } />
      </div>
    );
  }
}
