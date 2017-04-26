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

import { LinearProgress } from 'material-ui';
import React, { PropTypes } from 'react';

export default function Progress ({ className, color, determinate, max, min, style, value }) {
  return (
    <LinearProgress
      className={ className }
      color={ color }
      max={ max }
      min={ min }
      mode={
        determinate
          ? 'determinate'
          : 'indeterminate'
      }
      style={ style }
      value={ value }
    />
  );
}

Progress.propTypes = {
  className: PropTypes.string,
  color: PropTypes.string,
  determinate: PropTypes.bool,
  max: PropTypes.number,
  min: PropTypes.number,
  style: PropTypes.object,
  value: PropTypes.number
};

Progress.defaultProps = {
  determinate: false
};
