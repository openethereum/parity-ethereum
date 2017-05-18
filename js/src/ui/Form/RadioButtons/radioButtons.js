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
import { Radio as SemanticRadio } from 'semantic-ui-react';

import { arrayOrObjectProptype } from '@parity/shared/util/proptypes';

import LabelWrapper from '../LabelWrapper';
import Label from '../Label';

import styles from './radioButtons.css';

export default function RadioButtons ({ className, label, name, onChange, value, values }) {
  const _onChange = (event, { value }) => onChange(event, value);

  return (
    <LabelWrapper
      className={ `${styles.container} ${className}` }
      label={ label }
    >
      {
        values.map(({ description, key, label }) => (
          <div
            className={ styles.radioContainer }
            key={ key }
          >
            <SemanticRadio
              checked={ value === key }
              className={ styles.radio }
              label={
                <Label className={ styles.label }>
                  <div className={ styles.name }>
                    { label }
                  </div>
                  {
                    description && (
                      <div className={ styles.description }>
                        { description }
                      </div>
                    )
                  }
                </Label>
              }
              name={ name }
              onChange={ _onChange }
              value={ key }
            />
          </div>
        ))
      }
    </LabelWrapper>
  );
}

RadioButtons.propTypes = {
  className: PropTypes.string,
  label: PropTypes.node,
  name: PropTypes.string.isRequired,
  onChange: PropTypes.func.isRequired,
  value: PropTypes.any,
  values: arrayOrObjectProptype().isRequired
};
