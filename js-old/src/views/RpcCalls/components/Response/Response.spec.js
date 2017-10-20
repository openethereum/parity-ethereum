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

import React from 'react';
import { shallow } from 'enzyme';

import '../../../../environment/tests';

import Response from './Response';

describe('views/Status/components/Response', () => {
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
