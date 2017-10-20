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
import { Rectangle } from 'recharts';

import { COLORS, countModifier } from '../util';

export default class CustomCursor extends Component {
  static propTypes = {
    x: PropTypes.number,
    y: PropTypes.number,
    width: PropTypes.number,
    height: PropTypes.number,
    onClick: PropTypes.func,
    getIndex: PropTypes.func,
    counts: PropTypes.object,
    yDomain: PropTypes.array
  }

  render () {
    const { x, y, width, height, getIndex, counts, yDomain } = this.props;

    const index = getIndex();

    if (index === -1) {
      return null;
    }

    const count = countModifier(counts[index]);
    const barHeight = (count / yDomain[1]) * (y + height);

    return (
      <g>
        <Rectangle
          x={ x }
          y={ 0 }
          width={ width }
          height={ height + y }
          fill='transparent'
          onClick={ this.onClick }
        />
        <Rectangle
          x={ x }
          y={ y + (height - barHeight) }
          width={ width }
          height={ barHeight }
          fill={ COLORS.hover }
          onClick={ this.onClick }
        />
      </g>
    );
  }

  onClick = () => {
    const { onClick, getIndex } = this.props;
    const index = getIndex();

    onClick({ index });
  }
}
