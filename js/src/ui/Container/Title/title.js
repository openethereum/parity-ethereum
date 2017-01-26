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

import { nodeOrStringProptype } from '~/util/proptypes';

import styles from './title.css';

export default class Title extends Component {
  static propTypes = {
    byline: nodeOrStringProptype(),
    className: PropTypes.string,
    description: nodeOrStringProptype(),
    title: nodeOrStringProptype()
  }

  render () {
    const { byline, className, title } = this.props;

    const byLine = typeof byline === 'string'
      ? (
        <span title={ byline }>
          { byline }
        </span>
      )
      : byline;

    return (
      <div className={ className }>
        <h3 className={ styles.title }>
          { title }
        </h3>
        <div className={ styles.byline }>
          { byLine }
        </div>
        { this.renderDescription() }
      </div>
    );
  }

  renderDescription () {
    const { description } = this.props;

    if (!description) {
      return null;
    }

    const desc = typeof description === 'string'
      ? (
        <span title={ description }>
          { description }
        </span>
      )
      : description;

    return (
      <div className={ styles.description }>
        { desc }
      </div>
    );
  }
}
