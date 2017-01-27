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
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';

import { nodeOrStringProptype } from '~/util/proptypes';

import styles from './actionbar.css';

export default class Actionbar extends Component {
  static propTypes = {
    title: nodeOrStringProptype(),
    buttons: PropTypes.array,
    children: PropTypes.node,
    className: PropTypes.string
  };

  render () {
    const { children, className } = this.props;
    const classes = `${styles.actionbar} ${className}`;

    return (
      <Toolbar className={ classes }>
        { this.renderTitle() }
        { this.renderButtons() }
        { children }
      </Toolbar>
    );
  }

  renderButtons () {
    const { buttons } = this.props;

    if (!buttons || !buttons.length) {
      return null;
    }

    return (
      <ToolbarGroup className={ styles.toolbuttons }>
        { buttons }
      </ToolbarGroup>
    );
  }

  renderTitle () {
    const { title } = this.props;

    return (
      <h3 className={ styles.tooltitle }>
        { title }
      </h3>
    );
  }
}
