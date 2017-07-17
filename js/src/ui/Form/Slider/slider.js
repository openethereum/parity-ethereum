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

import React from 'react';
import PropTypes from 'prop-types';

import styles from './slider.css';

export default function Slider ({ className, max, min = 0, onChange, step = 1, value }) {
  const _onChange = (event) => onChange && onChange(event, event.target.value);

  return (
    <input
      className={ `${styles.slider} ${className}` }
      max={ max }
      min={ min }
      onChange={ _onChange }
      step={ step }
      type='range'
      value={ value }
    />
  );
}

Slider.propTypes = {
  className: PropTypes.string,
  max: PropTypes.number,
  min: PropTypes.number,
  onChange: PropTypes.func,
  step: PropTypes.number,
  value: PropTypes.number
};
