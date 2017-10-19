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

import PasswordStrength from './passwordStrength';

const INPUT_A = 'l33t_test';
const INPUT_B = 'Fu£dk;s$%kdlaOe9)_';
const INPUT_NULL = '';

function render (props) {
  return shallow(
    <PasswordStrength { ...props } />
  ).shallow();
}

describe('ui/Form/PasswordStrength', () => {
  describe('rendering', () => {
    it('renders', () => {
      expect(render({ input: INPUT_A })).to.be.ok;
    });

    it('renders a linear progress', () => {
      expect(render({ input: INPUT_A }).find('LinearProgress')).to.be.ok;
    });

    describe('compute strength', () => {
      it('has low score with empty input', () => {
        expect(
          render({ input: INPUT_NULL }).find('LinearProgress').props().value
        ).to.equal(20);
      });

      it('has medium score', () => {
        expect(
          render({ input: INPUT_A }).find('LinearProgress').props().value
        ).to.equal(60);
      });

      it('has high score', () => {
        expect(
          render({ input: INPUT_B }).find('LinearProgress').props().value
        ).to.equal(100);
      });
    });
  });
});
