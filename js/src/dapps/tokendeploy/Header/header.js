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

import layout from '../style.css';
import styles from './header.css';

const HEADERS = [
  { path: 'overview', className: styles.overview, title: 'Overview', byline: 'Displays all the current information relating to your own deployed tokens' },
  { path: 'send', className: styles.send, title: 'Send', byline: 'Send token associated with your accounts to other addresses' },
  { path: 'events', className: styles.events, title: 'Events', byline: 'Track the events for your tokens, showing actions as they hapenned' },
  { path: 'deploy', className: styles.deploy, title: 'Deploy', byline: 'Deploy a new token to the network' },
  { path: 'status', className: styles.status, title: 'Status', byline: 'Show the status of all network tokens deployed with this application' }
];

export default class Header extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  render () {
    const path = (window.location.hash || '').split('?')[0].split('/')[1];
    const offset = HEADERS.findIndex((header) => header.path === path);

    return (
      <table className={ styles.header }>
        <tbody>
          <tr>
            { this.renderHeader(0, offset) }
            { this.renderHeader(1, offset) }
            { this.renderHeader(2, offset) }
          </tr>
          <tr>
            { this.renderHeader(3, offset) }
            { this.renderHeader(4, offset) }
          </tr>
        </tbody>
      </table>
    );
  }

  renderHeader (index, offset) {
    const isFirst = index === 0;
    const position = index + offset;
    const header = HEADERS[position < HEADERS.length ? position : position - HEADERS.length];
    const classes = `${styles.nav} ${isFirst ? styles.navCurrent : styles.navNext} ${header.className}`;

    return (
      <td
        className={ classes }
        colSpan={ isFirst ? 2 : 1 }
        rowSpan={ isFirst ? 2 : 1 }
        onClick={ this.onNavigate(header.path) }>
        <div className={ layout.title }>
          { header.title }
        </div>
        <div className={ styles.byline }>
          { header.byline }
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
