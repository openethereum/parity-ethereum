// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { nodeOrStringProptype } from '~/util/proptypes';

import Title from './Title';

import styles from './container.css';

export default class Container extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    compact: PropTypes.bool,
    hover: PropTypes.node,
    light: PropTypes.bool,
    style: PropTypes.object,
    tabIndex: PropTypes.number,
    title: nodeOrStringProptype()
  }

  render () {
    const { className, light, style, tabIndex } = this.props;
    const props = {};

    if (Number.isInteger(tabIndex)) {
      props.tabIndex = tabIndex;
    }

    return (
      <div
        className={
          [
            styles.container,
            light
              ? styles.light
              : null,
            className
          ].join(' ')
        }
        style={ style }
        { ...props }
      >
        { this.renderBody() }
        { this.renderBodyHover() }
      </div>
    );
  }

  renderBody () {
    const { children, compact } = this.props;

    return (
      <Card
        className={
          compact
            ? styles.compact
            : styles.padded
        }
      >
        { this.renderTitle() }
        { children }
      </Card>
    );
  }

  renderBodyHover () {
    const { hover } = this.props;

    if (!hover) {
      return null;
    }

    return (
      <Card
        className={ styles.hover }
        data-hover='show'
      >
        { hover }
      </Card>
    );
  }

  renderTitle () {
    const { title } = this.props;

    if (!title) {
      return null;
    }

    return (
      <Title title={ title } />
    );
  }
}
