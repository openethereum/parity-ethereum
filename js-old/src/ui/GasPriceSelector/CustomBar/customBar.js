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

import { COLORS } from '../util';

export default class CustomBar extends Component {
  static propTypes = {
    selected: PropTypes.number,
    x: PropTypes.number,
    y: PropTypes.number,
    width: PropTypes.number,
    height: PropTypes.number,
    index: PropTypes.number,
    onClick: PropTypes.func
  }

  render () {
    const { x, y, selected, index, width, height, onClick } = this.props;

    const fill = selected === index
      ? COLORS.selected
      : COLORS.default;

    const borderWidth = 0.5;
    const borderColor = 'rgba(255, 255, 255, 0.5)';

    return (
      <g>
        <Rectangle
          x={ x - borderWidth }
          y={ y }
          width={ borderWidth }
          height={ height }
          fill={ borderColor }
        />
        <Rectangle
          x={ x + width }
          y={ y }
          width={ borderWidth }
          height={ height }
          fill={ borderColor }
        />
        <Rectangle
          x={ x - borderWidth }
          y={ y - borderWidth }
          width={ width + borderWidth * 2 }
          height={ borderWidth }
          fill={ borderColor }
        />
        <Rectangle
          x={ x }
          y={ y }
          width={ width }
          height={ height }
          fill={ fill }
          onClick={ onClick }
        />
      </g>
    );
  }
}
