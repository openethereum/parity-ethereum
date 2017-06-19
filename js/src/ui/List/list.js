import PropTypes from 'prop-types';
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
import { List as SemanticList } from 'semantic-ui-react';

import LabelWrapper from '../Form/LabelWrapper';
import Item from './Item';

import styles from './list.css';

export default function List ({ className, items, label, onClick, style }) {
  const wrapOnClick = (key) => {
    return (event) => onClick && onClick(event, key);
  };

  return (
    <LabelWrapper label={ label }>
      <SemanticList className={ `${styles.list} ${className}` }>
        {
          items.filter((item) => item).map(({ buttons, description, icon, isActive, key, label }, index) => (
            <Item
              buttons={ buttons }
              description={ description }
              icon={ icon }
              isActive={ isActive }
              key={ key || index }
              label={ label }
              onClick={ wrapOnClick(key || index) }
            />
          ))
        }
      </SemanticList>
    </LabelWrapper>
  );
}

List.Item = Item;

List.propTypes = {
  className: PropTypes.string,
  items: PropTypes.array,
  label: PropTypes.node,
  onClick: PropTypes.func,
  style: PropTypes.object
};
