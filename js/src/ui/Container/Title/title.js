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

import { CardTitle } from 'material-ui/Card';

import styles from './title.css';

const TITLE_STYLE = { textTransform: 'uppercase', padding: 0 };

export default class Title extends Component {
  static propTypes = {
    title: PropTypes.oneOfType([
      PropTypes.string, PropTypes.node
    ]),
    byline: PropTypes.string
  }

  state = {
    name: 'Unnamed'
  }

  render () {
    return (
      <div>
        <CardTitle
          style={ TITLE_STYLE }
          title={ this.props.title } />
        <div className={ styles.byline }>
          { this.props.byline }
        </div>
      </div>
    );
  }
}
