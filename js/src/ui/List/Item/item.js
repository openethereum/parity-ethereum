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
import { List as SemanticList } from 'semantic-ui-react';

import styles from './item.css';

export default function Item ({ buttons, className, description, icon, isActive, label, onClick, style }) {
  return (
    <SemanticList.Item
      className={ `${styles.item} ${isActive ? styles.active : styles.inactive} ${className}` }
      onClick={ onClick }
      style={ style }
    >
      {
        icon && (
          <SemanticList.Icon>
            { icon }
          </SemanticList.Icon>
        )
      }
      <SemanticList.Content>
        <div className={ styles.label }>
          { label }
        </div>
        <div className={ styles.description }>
          { description }
        </div>
        <div className={ styles.buttons }>
          { buttons }
        </div>
      </SemanticList.Content>
    </SemanticList.Item>
  );
}

Item.propTypes = {
  buttons: PropTypes.any,
  className: PropTypes.string,
  description: PropTypes.node,
  icon: PropTypes.node,
  isActive: PropTypes.bool,
  label: PropTypes.node,
  onClick: PropTypes.func,
  style: PropTypes.object
};
