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

import { shallow } from 'enzyme';
import React from 'react';

import Title from './';

let component;
let instance;

function render (props = {}) {
  component = shallow(
    <Title
      activeStep={ 0 }
      byline='testByline'
      className='testClass'
      description='testDescription'
      title='testTitle'
      { ...props }
    />
  );
  instance = component.instance();

  return component;
}

describe('ui/Title', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('instance methods', () => {
    describe('renderSteps', () => {
      let stepper;

      beforeEach(() => {
        render({ steps: ['stepA', 'stepB'] });
        stepper = shallow(instance.renderSteps());
      });

      it('renders the Stepper', () => {
        expect(stepper.find('Stepper').get(0)).to.be.ok;
      });
    });

    describe('renderTimeline', () => {
      let steps;

      beforeEach(() => {
        render({ steps: ['stepA', 'StepB'] });
        steps = instance.renderTimeline();
      });

      it('renders the Step', () => {
        expect(steps.length).to.equal(2);
      });
    });

    describe('renderWaiting', () => {
      let waiting;

      beforeEach(() => {
        render({ busy: true });
        waiting = shallow(instance.renderWaiting());
      });

      it('renders the LinearProgress', () => {
        expect(waiting.find('LinearProgress').get(0)).to.be.ok;
      });
    });
  });
});
