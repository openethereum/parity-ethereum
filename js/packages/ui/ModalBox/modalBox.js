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

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';

import styles from './modalBox.css';

export default function ModalBox ({ children, icon, summary }) {
  return (
    <div className={ styles.body }>
      <div className={ styles.icon }>
        { icon }
      </div>
      <div className={ styles.content }>
        {
          summary && (
            <div className={ styles.summary }>
              { summary }
            </div>
          )
        }
        {
          children && (
            <div className={ styles.body }>
              { children }
            </div>
          )
        }
      </div>
    </div>
  );
}

ModalBox.propTypes = {
  children: PropTypes.node,
  icon: PropTypes.node.isRequired,
  summary: nodeOrStringProptype()
};
