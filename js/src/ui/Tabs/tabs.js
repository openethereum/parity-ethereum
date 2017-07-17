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
import PropTypes from 'prop-types';
import { Menu as SemanticMenu } from 'semantic-ui-react';

import Tab from './Tab';

export default function Tabs ({ activeTab, className, tabs, onChange }) {
  const onTabClick = (event, { index }) => onChange && onChange(event, index);

  return (
    <SemanticMenu
      className={ className }
      pointing
    >
      {
        tabs.filter((tab) => tab).map((tab, index) => (
          <Tab
            isActive={ activeTab === index }
            index={ index }
            key={ `tab_${index}` }
            label={ tab.label || tab }
            onClick={ onTabClick }
          />
        ))
      }
    </SemanticMenu>
  );
}

Tabs.Tab = Tab;

Tabs.propTypes = {
  activeTab: PropTypes.number,
  className: PropTypes.string,
  onChange: PropTypes.func,
  tabs: PropTypes.array
};
