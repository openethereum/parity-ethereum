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
import { FormattedMessage } from 'react-intl';

import styles from '../firstRun.css';

export default function Completed () {
  return (
    <div className={ styles.completed }>
      <p>
        <FormattedMessage
          id='firstRun.completed.congrats'
          defaultMessage='Congratulations! Your node setup has been completed successfully and you are ready to use the application.'
        />
      </p>
      <p>
        <FormattedMessage
          id='firstRun.completed.next'
          defaultMessage='Next you will receive a walk-through of the available functions and the general application interface to get you up and running in record time.'
        />
      </p>
    </div>
  );
}
