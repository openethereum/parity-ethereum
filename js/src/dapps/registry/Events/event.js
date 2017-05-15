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

import moment from 'moment';
import React, { PropTypes } from 'react';

import Address from '../ui/address';
import ApplicationStore from '../Application/application.store';
import Hash from '../ui/hash';

import styles from './event.css';

const { api } = ApplicationStore.get();

const Param = ({ data, label }) => {
  if (!data) {
    return null;
  }

  const { value } = data;
  const display = value && typeof value.peek === 'function'
    ? api.util.bytesToHex(value.peek())
    : value;

  return (
    <div className={ styles.param }>
      <span className={ styles.label }>
        { label }
      </span>
      <code>
        { display }
      </code>
    </div>
  );
};

Param.propTypes = {
  label: PropTypes.string.isRequired,
  data: PropTypes.object
};

const Event = ({ event }) => {
  const { state, timestamp, transactionHash, type, parameters, from } = event;
  const isPending = state === 'pending';

  const { reverse, owner, name, plainKey } = parameters;
  const sender = (reverse && reverse.value) ||
    (owner && owner.value) ||
    from;

  const classes = [];

  if (isPending) {
    classes.push(styles.pending);
  }

  return (
    <div className={ classes.join(' ') }>
      <div className={ styles.date }>
        {
          isPending
          ? '(pending)'
          : moment(timestamp).fromNow()
        }
      </div>
      <div className={ styles.infoContainer }>
        <Address
          address={ sender }
          className={ styles.address }
        />
        <div className={ styles.transaction }>
          <span className={ styles.event }>{ type }</span>
          <span className={ styles.arrow }>→</span>
          <Hash
            hash={ transactionHash }
            linked
          />
        </div>
        <div className={ styles.params }>
          <Param
            data={ name }
            label='Name'
          />
          <Param
            data={ plainKey }
            label='Key'
          />
        </div>
      </div>
    </div>
  );
};

Event.propTypes = {
  event: PropTypes.object
};

export default Event;
