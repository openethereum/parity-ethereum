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
import React, { PropTypes } from 'react';
import sinon from 'sinon';

import muiTheme from '../Theme';

import ConfirmDialog from './';

let component;
let instance;
let onConfirm;
let onDeny;

function createRedux () {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        settings: {
          backgroundSeed: 'xyz'
        }
      };
    }
  };
}

function render (props = {}) {
  onConfirm = sinon.stub();
  onDeny = sinon.stub();

  if (props.visible === undefined) {
    props.visible = true;
  }

  const baseComponent = shallow(
    <ConfirmDialog
      { ...props }
      title='test title'
      onConfirm={ onConfirm }
      onDeny={ onDeny }
    >
      <div id='testContent'>
        some test content
      </div>
    </ConfirmDialog>
  );

  instance = baseComponent.instance();
  component = baseComponent.find('Connect(Modal)').shallow({
    childContextTypes: {
      muiTheme: PropTypes.object,
      store: PropTypes.object
    },
    context: {
      muiTheme,
      store: createRedux()
    }
  });

  return component;
}

describe('ui/ConfirmDialog', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('renders the body as provided', () => {
    expect(render().find('div[id="testContent"]').text()).to.equal('some test content');
  });

  describe('properties', () => {
    let props;

    beforeEach(() => {
      props = render().props();
    });

    it('passes the actions', () => {
      expect(props.actions).to.deep.equal(instance.renderActions());
    });

    it('passes title', () => {
      expect(props.title).to.equal('test title');
    });

    it('passes visiblity flag', () => {
      expect(props.visible).to.be.true;
    });
  });

  describe('renderActions', () => {
    describe('defaults', () => {
      let buttons;

      beforeEach(() => {
        render();
        buttons = instance.renderActions();
      });

      it('renders with supplied onConfim/onDeny callbacks', () => {
        expect(buttons[0].props.onClick).to.deep.equal(onDeny);
        expect(buttons[1].props.onClick).to.deep.equal(onConfirm);
      });

      it('renders default labels', () => {
        expect(buttons[0].props.label.props.id).to.equal('ui.confirmDialog.no');
        expect(buttons[1].props.label.props.id).to.equal('ui.confirmDialog.yes');
      });

      it('renders default icons', () => {
        expect(buttons[0].props.icon.type.displayName).to.equal('ContentClear');
        expect(buttons[1].props.icon.type.displayName).to.equal('NavigationCheck');
      });
    });

    describe('overrides', () => {
      let buttons;

      beforeEach(() => {
        render({
          labelConfirm: 'labelConfirm',
          labelDeny: 'labelDeny',
          iconConfirm: 'iconConfirm',
          iconDeny: 'iconDeny'
        });
        buttons = instance.renderActions();
      });

      it('renders supplied labels', () => {
        expect(buttons[0].props.label).to.equal('labelDeny');
        expect(buttons[1].props.label).to.equal('labelConfirm');
      });

      it('renders supplied icons', () => {
        expect(buttons[0].props.icon).to.equal('iconDeny');
        expect(buttons[1].props.icon).to.equal('iconConfirm');
      });
    });
  });
});
