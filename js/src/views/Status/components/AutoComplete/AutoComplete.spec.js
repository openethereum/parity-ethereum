import React from 'react';
import { shallow } from 'enzyme';

import getMuiTheme from 'material-ui/styles/getMuiTheme';

import WrappedAutoComplete from './AutoComplete';

describe('components/AutoComplete', () => {
  describe('rendering', () => {
    let rendered;

    beforeEach(() => {
      const dataSource = ['abc', 'def', 'ghi'];
      const component =
        <WrappedAutoComplete
          dataSource={ dataSource }
          name='testComponent'
        />;

      rendered = shallow(component, { context: { muiTheme: getMuiTheme({}) } });
    });

    it('renders the material AutoComplete component', () => {
      expect(rendered).to.be.ok;
      expect(rendered).to.have.exactly(1).descendants('AutoComplete');
    });
  });
});
