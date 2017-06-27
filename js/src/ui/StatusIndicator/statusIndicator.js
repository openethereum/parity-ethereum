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
import ReactTooltip from 'react-tooltip';

import styles from './statusIndicator.css';


const statuses = ['bad', 'needsAttention', 'ok'];

export default class StatusIndicator extends Component {
  static propTypes = {
    type: PropTypes.oneOf(['radial', 'signal']),
    id: PropTypes.string.isRequired,
    status: PropTypes.oneOf(statuses).isRequired,
    title: PropTypes.string
  };

  static defaultProps = {
    type: 'signal',
    title: ''
  };

  render () {
    const { id, status, title, type } = this.props;
    return (
      <span className={ styles.status }>
        <span className={ `${styles[type]} ${styles[status]}` }
          data-tip
          data-for={ `status-${id}` }
          data-effect='solid'
        >
          { type === 'signal' && statuses.map(this.renderBar) }
        </span>
        <ReactTooltip id={ `status-${id}` }>
          { title || 'All OK' }
        </ReactTooltip>
      </span>
    );
  }

  renderBar = (signal) => {
    const idx = statuses.indexOf(this.props.status);
    const isActive = statuses.indexOf(signal) <= idx;
    const activeClass = isActive ? styles.active : '';

    return (
      <span className={ `${styles.bar} ${styles[signal]} ${activeClass}` } />
    );
  }
}
