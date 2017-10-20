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

import StatusIndicator from '@parity/ui/StatusIndicator';

import styles from './syncWarning.css';

function SyncWarning ({ className }, { api }) {
  const statusStore = StatusIndicator.Store.get(api);
  const isOk = !statusStore.health.overall || (!statusStore.health.overall.isNotReadyYet && statusStore.health.overall.status === 'ok');

  if (isOk) {
    return null;
  }

  return (
    <div className={ className }>
      <div className={ styles.body }>
        {
          statusStore.health.overall.message.map((message) => (
            <p key={ message }>
              { message }
            </p>
          ))
        }
      </div>
    </div>
  );
}

SyncWarning.contextTypes = {
  api: PropTypes.object
};

SyncWarning.propTypes = {
  className: PropTypes.string
};

export default observer(SyncWarning);
