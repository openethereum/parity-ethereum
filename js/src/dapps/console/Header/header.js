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

import ApplicationStore from '../Application/application.store';

import styles from './header.css';

@observer
export default class Header extends Component {
  application = ApplicationStore.get();

  render () {
    return (
      <div className={ styles.container }>
        <div className={ styles.tabs }>
          { this.renderTabs() }
        </div>
      </div>
    );
  }

  renderTabs () {
    const { view } = this.application;

    return this.application.views.map((tab) => {
      const { label, id } = tab;
      const classes = [ styles.tab ];
      const onClick = () => this.handleClickTab(id);

      if (id === view) {
        classes.push(styles.active);
      }

      return (
        <div
          className={ classes.join(' ') }
          key={ id }
          onClick={ onClick }
        >
          { label }
        </div>
      );
    });
  }

  handleClickTab = (id) => {
    this.application.setView(id);
  };
}
