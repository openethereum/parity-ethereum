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
import { observer } from 'mobx-react';
import moment from 'moment';

import Store from '../BlockNumber/store';

import styles from './blockTimestamp.css';

function BlockTimestamp ({ className }, { api }) {
  const store = Store.get(api);

  if (!store.blockTimestamp) {
    return null;
  }

  return (
    <div className={ [styles.blockTimestamp, className].join(' ') }>
      { moment(store.blockTimestamp).calendar() }
    </div>
  );
}

BlockTimestamp.propTypes = {
  className: PropTypes.string
};

BlockTimestamp.contextTypes = {
  api: PropTypes.object.isRequired
};

const ObserverComponent = observer(BlockTimestamp);

ObserverComponent.Store = Store;

export default ObserverComponent;
