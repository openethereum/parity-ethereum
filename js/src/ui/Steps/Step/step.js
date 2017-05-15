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
import { Step as SemanticStep } from 'semantic-ui-react';

export default function Step ({ className, isActive, isCompleted, label }) {
  return (
    <SemanticStep
      className={ className }
      completed={ isCompleted }
      active={ isActive }
    >
      <SemanticStep.Content>
        <SemanticStep.Title>
          { label }
        </SemanticStep.Title>
      </SemanticStep.Content>
    </SemanticStep>
  );
}

Step.propTypes = {
  className: PropTypes.string,
  isActive: PropTypes.bool,
  isCompleted: PropTypes.bool,
  label: PropTypes.node
};
