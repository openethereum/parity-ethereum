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

import React, { PropTypes } from 'react';
import DateTime from 'react-datetime';

import { parseI18NString } from '@parity/shared/util/messages';

import LabelComponent from '~/ui/Form/LabelComponent';

import styles from './inputDateTime.css';

import 'react-datetime/css/react-datetime.css';

export default function InputDateTime ({ className, hint, label, onChange, value }, context) {
  const _onChange = (value) => onChange && onChange(null, value);

  return (
    <LabelComponent
      className={ `${styles.container} ${className}` }
      label={ label }
    >
      <div className='ui fluid input'>
        <DateTime
          className={ styles.input }
          inputProps={ {
            placeholder: parseI18NString(context, hint)
          } }
          onChange={ _onChange }
          value={ value }
        />
      </div>
    </LabelComponent>
  );
}

InputDateTime.contextTypes = {
  intl: PropTypes.object
};

InputDateTime.propTypes = {
  className: PropTypes.string,
  hint: PropTypes.node,
  label: PropTypes.node,
  onChange: PropTypes.func,
  value: PropTypes.object.isRequired
};
