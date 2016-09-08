import React from 'react';
import { shallow } from 'enzyme';

import '../../env-specific/tests';

import Response from './Response';

describe('components/Response', () => {
  describe('rendering', () => {
    it('renders non-arrays/non-objects exactly as received', () => {
      const TEST = '1234567890';
      const rendered = shallow(<Response response={ TEST } />);

      expect(rendered).to.have.html(`<pre>${TEST}</pre>`);
    });

    it('renders arrays properly with index and value', () => {
      const TEST = ['123', '456'];
      const rendered = shallow(<Response response={ TEST } />);

      expect(rendered).to.have.html('<pre><span>[123</span><span>,<br/>456]</span></pre>');
    });

    it('renders objects properly with key and value', () => {
      const TEST = { foo: '123', bar: '456' };
      const rendered = shallow(<Response response={ TEST } />);

      expect(rendered).to.have.html('<pre><span>{</span><span> &quot;foo&quot;: &quot;123&quot;,<br/></span><span> &quot;bar&quot;: &quot;456&quot;<br/></span><span>}</span></pre>');
    });
  });
});
