import React from 'react';
import { shallow } from 'enzyme';

import Box from './Box';

describe('components/Box', () => {
  describe('rendering', () => {
    const title = 'test title';
    let rendered;

    beforeEach(() => {
      rendered = shallow(<Box title={ title } />);
    });

    it('renders the component', () => {
      expect(rendered).to.be.ok;
      expect(rendered).to.have.className('dapp-box');
    });

    it('renders the title', () => {
      expect(rendered.find('h2')).to.have.text(title);
    });

    it('renders no default value', () => {
      expect(rendered).to.not.have.descendants('h1');
    });
  });

  describe('contents', () => {
    const value = 'test value';
    const child = 'this is the child value';

    let rendered;

    beforeEach(() => {
      rendered = shallow(
        <Box
          title='title'
          value={ value }
        >
          <pre>{ child }</pre>
        </Box>
      );
    });

    it('renders the value', () => {
      expect(rendered.find('h1')).to.have.text(value);
    });

    it('wraps the children', () => {
      expect(rendered.find('pre')).to.have.text(child);
    });
  });
});
