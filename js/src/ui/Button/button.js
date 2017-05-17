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
import { FlatButton } from 'material-ui';

import { nodeOrStringProptype } from '~/util/proptypes';

import styles from './button.css';

let id = 0;

export default class Button extends Component {
  static propTypes = {
    backgroundColor: PropTypes.string,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    icon: PropTypes.node,
    label: nodeOrStringProptype(),
    onClick: PropTypes.func,
    primary: PropTypes.bool,
    tooltip: PropTypes.bool
  };

  static defaultProps = {
    primary: true,
    tooltip: false
  };

  componentWillMount () {
    this.id = id++;
  }

  render () {
    const { className, backgroundColor, disabled, icon, label, primary, onClick, tooltip } = this.props;
    const button = (
      <FlatButton
        backgroundColor={ backgroundColor }
        className={ className }
        disabled={ disabled }
        icon={ icon }
        label={ label }
        onTouchTap={ onClick }
        primary={ primary }
      />
    );

    if (!tooltip) {
      return button;
    }

    return (
      <div>
        <div
          data-tip
          data-for={ `button_${this.id}` }
          data-effect='solid'
          data-place='bottom'
        >
          { button }
        </div>
        <ReactTooltip id={ `button_${this.id}` }>
          <div className={ styles.tooltip }>
            { label }
          </div>
        </ReactTooltip>
      </div>
    );
  }
}
