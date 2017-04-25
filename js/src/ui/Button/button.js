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

import styles from './button.css';

export default class Button extends Component {
  static propTypes = {
    backgroundColor: PropTypes.string,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    icon: PropTypes.node,
    label: nodeOrStringProptype(),
    onClick: PropTypes.func,
    primary: PropTypes.bool,
    before: nodeOrStringProptype(),
    after: nodeOrStringProptype()
  }

  static defaultProps = {
    primary: true
  }

  render () {
    let {
      className,
      backgroundColor,
      disabled,
      icon,
      label,
      onClick,
      before,
      after
    } = this.props;

    let style = { backgroundColor };

    if (className) {
      className = `${className} ${styles.customButton}`;
    } else {
      className = styles.customButton;
    }
    if (disabled) {
      style = {
        userSelect: 'none',
        pointerEvents: 'none',
        color: 'rgba(255, 255, 255, 0.298039)',
        cursor: 'default'
      };
    }
    if (icon) {
      before = icon;
    }
    if (before && (!label)) {
      label = before;
      before = null;
    }

    return (
      <div className={ className } style={ style } onClick={ onClick }>
        <div>
          <span id={ styles.positionBefore }>
            { before }
          </span>
          <span id={ styles.content }>
            { label }
          </span>
          <span id={ styles.positionAfter }>
            { after }
          </span>
        </div>
      </div>
    );
  }
}
