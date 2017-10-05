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
import { connect } from 'react-redux';

import StatusIndicator from '@parity/ui/StatusIndicator';

import styles from './syncWarning.css';

function SyncWarning ({ className, isOk, health }) {
  console.log('SyncWarning', isOk, health);

  if (isOk) {
    return null;
  }

  return (
    <div className={ className }>
      <div className={ styles.body }>
        {
          health.overall.message.map((message) => (
            <p key={ message }>
              { message }
            </p>
          ))
        }
      </div>
    </div>
  );
}

SyncWarning.propTypes = {
  className: PropTypes.string,
  isOk: PropTypes.bool.isRequired,
  health: PropTypes.object.isRequired
};

function mapStateToProps (state) {
  const { health } = state.nodeStatus;
  const isNotAvailableYet = health.overall.isNotReady;
  const isOk = !isNotAvailableYet && health.overall.status === 'ok';

  return {
    isOk,
    health
  };
}

export default connect(
  mapStateToProps,
  null
)(SyncWarning);
