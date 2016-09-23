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

export default class ParityBackground extends Component {
  static contextTypes = {
    muiTheme: PropTypes.object.isRequired
  }

  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    gradient: PropTypes.string
  }

  render () {
    const { children, className, gradient } = this.props;
    const { muiTheme } = this.context;

    return (
      <div className={ className } style={ muiTheme.parity.getBackgroundStyle(gradient) }>
        { children }
      </div>
    );
  }
}
