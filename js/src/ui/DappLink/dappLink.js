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

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';

import styles from './dappLink.css';

export function topNavigate (to) {
  window.parent.location.hash = to;
}

export default class DappLink extends Component {
  static propTypes = {
    children: nodeOrStringProptype().isRequired,
    className: PropTypes.string,
    to: PropTypes.string.isRequired
  };

  render () {
    const { children, className } = this.props;

    return (
      <div
        className={ [styles.link, className].join(' ') }
        onClick={ this.onClick }
      >
        { children }
      </div>
    );
  }

  onClick = () => {
    topNavigate(this.props.to);
  }
}
