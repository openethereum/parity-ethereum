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
import ReactTooltip from 'react-tooltip';
import { observer } from 'mobx-react';

import Store from './store';

import styles from './statusIndicator.css';

const statuses = ['bad', 'needsAttention', 'ok'];

function StatusIndicator ({ id, status, title = [], tooltipPlacement, type = 'signal' }, { api }) {
  const store = Store.get(api);
  const checkStatus = status || store.overall.status;
  const message = title.length
    ? title
    : store.overall.message;

  return (
    <span className={ styles.status }>
      <span
        className={ `${styles[type]} ${styles[checkStatus]}` }
        data-tip={ message.length }
        data-for={ `status-${id}` }
        data-place={ tooltipPlacement }
        data-effect='solid'
      >
        {
          type === 'signal'
            ? statuses.map((signal) => {
              const index = statuses.indexOf(checkStatus);
              const isActive = statuses.indexOf(signal) <= index;

              return (
                <span
                  key={ signal }
                  className={ `${styles.bar} ${styles[signal]} ${isActive ? styles.active : ''}` }
                />
              );
            })
            : null
        }
      </span>
      {
        message.find((x) => !x.isEmpty)
          ? (
            <ReactTooltip id={ `status-${id}` }>
              {
                message.map((x) => (
                  <div key={ x }>
                    { x }
                  </div>)
                )
              }
            </ReactTooltip>
          )
          : null
      }
    </span>
  );
}

StatusIndicator.propTypes = {
  type: PropTypes.oneOf([
    'radial', 'signal'
  ]),
  id: PropTypes.string.isRequired,
  status: PropTypes.oneOf(statuses),
  title: PropTypes.arrayOf(PropTypes.node),
  tooltipPlacement: PropTypes.oneOf([
    'left', 'top', 'bottom', 'right'
  ])
};

StatusIndicator.contextTypes = {
  api: PropTypes.object.isRequired
};

const ObserverComponent = observer(StatusIndicator);

ObserverComponent.Store = Store;

export default ObserverComponent;
