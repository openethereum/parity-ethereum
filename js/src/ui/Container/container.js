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
import { Card } from 'material-ui/Card';

import styles from './container.css';

export default class Container extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    compact: PropTypes.bool,
    light: PropTypes.bool,
    style: PropTypes.object
  }

  render () {
    const { children, className, compact, light, style } = this.props;
    const classes = `${styles.container} ${light ? styles.light : ''} ${className}`;

    return (
      <div className={ classes } style={ style }>
        <Card className={ compact ? styles.compact : styles.padded }>
          { children }
        </Card>
      </div>
    );
  }
}
