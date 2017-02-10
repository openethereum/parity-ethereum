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

import PAGES from '../pages';
import styles from './header.css';

export default class Header extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  render () {
    const path = (window.location.hash || '').split('?')[0].split('/')[1];
    const offset = PAGES.findIndex((header) => header.path === path);

    return (
      <div className={ styles.header }>
        <table className={ styles.navigation }>
          <tbody>
            <tr>
              { this.renderHeader(0, offset) }
              { this.renderHeader(1, offset) }
            </tr>
            <tr>
              { this.renderHeader(2, offset) }
            </tr>
          </tbody>
        </table>
      </div>
    );
  }

  renderHeader (position, offset) {
    const index = (position + offset) % PAGES.length;
    const page = PAGES[index];
    const background = `rgba(102, 34, 34, ${1 - (0.1 * position)})`;

    return (
      <td
        className={ position ? styles.navNext : styles.navCurrent }
        style={ { background } }
        colSpan={ position ? 1 : 2 }
        rowSpan={ position ? 1 : 2 }
        onClick={ this.onNavigate(page.path) }
      >
        <div className={ styles.title }>
          { page.title }
        </div>
        <div className={ styles.byline }>
          { page.byline }
        </div>
        <div className={ styles.description }>
          { position ? null : page.description }
        </div>
      </td>
    );
  }

  onNavigate = (route) => {
    const { router } = this.context;

    return (event) => {
      router.push(`/${route}`);
    };
  }
}
