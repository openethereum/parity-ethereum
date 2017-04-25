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
import { Button as SemButton } from 'semantic-ui-react';

import { nodeOrStringProptype } from '~/util/proptypes';

export default class Button extends Component {
  static propTypes = {
    active: PropTypes.bool,
    animated: PropTypes.bool,
    basic: PropTypes.bool,
    backgroundColor: PropTypes.string,
    className: PropTypes.string,
    color: PropTypes.string,
    disabled: PropTypes.bool,
    icon: PropTypes.node,
    label: nodeOrStringProptype(),
    onClick: PropTypes.func,
    primary: PropTypes.bool,
    size: PropTypes.string,
    toggle: PropTypes.bool
  }

  static defaultProps = {
    primary: true
  }

  render () {
    const {
      active,
      animated,
      basic,
      className,
      color,
      disabled,
      icon,
      label,
      onClick,
      primary,
      size,
      toggle
    } = this.props;

    return (
      <SemButton
        active={ active }
        animated={ animated }
        basic={ basic }
        className={ className }
        color={ color }
        disabled={ disabled }
        icon={ icon }
        content={ label }
        onClick={ onClick }
        primary={ primary }
        size={ size }
        toggle={ toggle }
      />
    );
  }
}
