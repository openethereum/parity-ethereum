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

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';

import styles from './actionbar.css';

export default function Actionbar ({ buttons, children, className, title }) {
  return (
    <div className={ `${styles.actionbar} ${className}` }>
      <h3 className={ styles.title }>
        { title }
      </h3>
      <div className={ styles.children }>
        { children }
      </div>
      <div className={ styles.buttons }>
        { buttons }
      </div>
    </div>
  );
}

Actionbar.propTypes = {
  title: nodeOrStringProptype(),
  buttons: PropTypes.array,
  children: PropTypes.node,
  className: PropTypes.string
};
