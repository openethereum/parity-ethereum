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

import FileSelect from './';

const FILE = {
  content: 'some test content',
  name: 'someName'
};

let component;
let globalFileReader;
let instance;
let onSelect;
let processedFile;

function stubReader () {
  globalFileReader = global.FileReader;

  global.FileReader = class {
    readAsText (file) {
      processedFile = file;

      this.onload({
        target: {
          result: file.content
        }
      });
    }
  };
}

function restoreReader () {
  global.FileReader = globalFileReader;
}

function render (props = {}) {
  onSelect = sinon.stub();
  component = shallow(
    <FileSelect
      onSelect={ onSelect }
      { ...props }
    />
  );
  instance = component.instance();

  return component;
}

describe('ui/Form/FileSelect', () => {
  beforeEach(() => {
    stubReader();
    render();
  });

  afterEach(() => {
    restoreReader();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('DropZone', () => {
    let label;
    let zone;

    beforeEach(() => {
      label = component.find('FormattedMessage');
      zone = component.find('Dropzone');
    });

    it('renders the label', () => {
      expect(label.props().id).to.equal('ui.fileSelect.defaultLabel');
    });

    it('attaches the onDrop event', () => {
      expect(zone.props().onDrop).to.equal(instance.onDrop);
    });

    it('does not allow multiples', () => {
      expect(zone.props().multiple).to.be.false;
    });
  });

  describe('event methods', () => {
    describe('onDrop', () => {
      beforeEach(() => {
        instance.onDrop([ FILE ]);
      });

      it('reads the file as dropped', () => {
        expect(processedFile).to.deep.equal(FILE);
      });

      it('calls prop onSelect with file & content', () => {
        expect(onSelect).to.have.been.calledWith(FILE.name, FILE.content);
      });
    });
  });
});
