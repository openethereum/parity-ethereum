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

import Step from './Step';

export default function Steps ({ activeStep, className, steps }) {
  if (!steps || steps.length < 2) {
    return null;
  }

  return (
    <div className={ className }>
      <SemanticStep.Group ordered>
        {
          steps.filter((step) => step).map((step, index) => (
            <Step
              isActive={ activeStep === index }
              isCompleted={ activeStep > index }
              key={ step.key || index }
              label={ step.label || step }
            />
          ))
        }
      </SemanticStep.Group>
    </div>
  );
}

Steps.Step = Step;

Steps.propTypes = {
  activeStep: PropTypes.number,
  className: PropTypes.string,
  steps: PropTypes.array
};
