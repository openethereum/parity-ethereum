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
import { Step, Stepper, StepLabel } from 'material-ui/Stepper';
import React, { Component, PropTypes } from 'react';

// TODO: It would make sense (going forward) to replace all uses of
// ContainerTitle with this component. In that case the styles for the
// h3 (title) can be pulled from there. (As it stands the duplication
// between the 2 has been removed, but as a short-term DRY only)
import { Title as ContainerTitle } from '~/ui/Container';
import { nodeOrStringProptype } from '~/util/proptypes';

import styles from './title.css';

export default class Title extends Component {
  static propTypes = {
    activeStep: PropTypes.number,
    description: nodeOrStringProptype(),
    busy: PropTypes.bool,
    busySteps: PropTypes.array,
    byline: nodeOrStringProptype(),
    className: PropTypes.string,
    isSubTitle: PropTypes.bool,
    steps: PropTypes.array,
    title: nodeOrStringProptype()
  }

  render () {
    const { activeStep, byline, className, description, isSubTitle, steps, title } = this.props;

    if (!title && !steps) {
      return null;
    }

    return (
      <div
        className={
          [
            isSubTitle
              ? styles.subtitle
              : styles.title,
            className
          ].join(' ')
        }
      >
        <ContainerTitle
          byline={ byline }
          description={ description }
          title={
            steps
              ? steps[activeStep || 0]
              : title
          }
        />
        { this.renderSteps() }
        { this.renderWaiting() }
      </div>
    );
  }

  renderSteps () {
    const { activeStep, steps } = this.props;

    if (!steps) {
      return;
    }

    return (
      <div className={ styles.steps }>
        <Stepper activeStep={ activeStep }>
          { this.renderTimeline() }
        </Stepper>
      </div>
    );
  }

  renderTimeline () {
    const { steps } = this.props;

    return steps.map((label, index) => {
      return (
        <Step key={ label.key || index }>
          <StepLabel>
            { label }
          </StepLabel>
        </Step>
      );
    });
  }

  renderWaiting () {
    const { activeStep, busy, busySteps } = this.props;
    const isWaiting = busy || (busySteps || []).includes(activeStep);

    if (!isWaiting) {
      return null;
    }

    return (
      <div className={ styles.waiting }>
        <LinearProgress />
      </div>
    );
  }
}
