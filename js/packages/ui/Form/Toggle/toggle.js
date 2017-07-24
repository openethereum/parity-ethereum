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
import { Radio as SemanticRadio } from 'semantic-ui-react';

import LabelWrapper from '../LabelWrapper';

export default function Toggle ({ className, label, onToggle, style, toggled }) {
  return (
    <LabelWrapper label={ label }>
      <SemanticRadio
        checked={ toggled }
        className={ className }
        onChange={ onToggle }
        style={ style }
        toggle
      />
    </LabelWrapper>
  );
}

Toggle.propTypes = {
  className: PropTypes.string,
  label: PropTypes.node,
  onToggle: PropTypes.func,
  style: PropTypes.object,
  toggled: PropTypes.bool
};
