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

import React, { PropTypes } from 'react';
import { Menu } from 'semantic-ui-react';

export default function Tabs ({ activeTab, className, tabs, onChange }) {
  const onTabClick = (event, { index }) => onChange(event, index);

  return (
    <Menu
      className={ className }
      pointing
    >
      {
        tabs.map((tab, index) => {
          if (!tab) {
            return null;
          }

          const key = `tab_${index}`;

          return (
            <Menu.Item
              active={ activeTab === index }
              index={ index }
              key={ key }
              name={ key }
              onClick={ onTabClick }
            >
              { tab.label || tab }
            </Menu.Item>
          );
        })
      }
    </Menu>
  );
}

Tabs.propTypes = {
  activeTab: PropTypes.number,
  className: PropTypes.string,
  onChange: PropTypes.func,
  tabs: PropTypes.array
};
