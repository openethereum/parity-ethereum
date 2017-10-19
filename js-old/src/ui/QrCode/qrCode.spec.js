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
import sinon from 'sinon';

import QrCode from './';

const DEFAULT_PROPS = {
  margin: 1,
  size: 4,
  value: 'someTestValue'
};

let component;
let instance;

function render (props = {}) {
  component = shallow(
    <QrCode
      { ...DEFAULT_PROPS }
      { ...props }
    />
  );
  instance = component.instance();

  return component;
}

describe('ui/QrCode', () => {
  beforeEach(() => {
    render();
    sinon.spy(instance, 'generateCode');
  });

  afterEach(() => {
    instance.generateCode.restore();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('lifecycle', () => {
    describe('componentWillMount', () => {
      it('generates the image on mount', () => {
        instance.componentWillMount();
        expect(instance.generateCode).to.have.been.calledWith(DEFAULT_PROPS);
      });
    });

    describe('componentWillReceiveProps', () => {
      it('does not re-generate when no props changed', () => {
        instance.componentWillReceiveProps(DEFAULT_PROPS);
        expect(instance.generateCode).not.to.have.been.called;
      });

      it('does not re-generate when className changed', () => {
        const nextProps = Object.assign({}, DEFAULT_PROPS, { className: 'test' });

        instance.componentWillReceiveProps(nextProps);
        expect(instance.generateCode).not.to.have.been.called;
      });

      it('does not re-generate when additional property changed', () => {
        const nextProps = Object.assign({}, DEFAULT_PROPS, { something: 'test' });

        instance.componentWillReceiveProps(nextProps);
        expect(instance.generateCode).not.to.have.been.called;
      });

      it('does re-generate when value changed', () => {
        const nextProps = Object.assign({}, DEFAULT_PROPS, { value: 'somethingElse' });

        instance.componentWillReceiveProps(nextProps);
        expect(instance.generateCode).to.have.been.calledWith(nextProps);
      });

      it('does re-generate when size changed', () => {
        const nextProps = Object.assign({}, DEFAULT_PROPS, { size: 10 });

        instance.componentWillReceiveProps(nextProps);
        expect(instance.generateCode).to.have.been.calledWith(nextProps);
      });

      it('does re-generate when margin changed', () => {
        const nextProps = Object.assign({}, DEFAULT_PROPS, { margin: 10 });

        instance.componentWillReceiveProps(nextProps);
        expect(instance.generateCode).to.have.been.calledWith(nextProps);
      });
    });
  });
});
