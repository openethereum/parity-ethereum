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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import ReactMarkdown from 'react-markdown';
import { Checkbox } from 'material-ui';

import styles from '../firstRun.css';

let tnc = '';

if (process.env.NODE_ENV !== 'test') {
  tnc = require('./tnc.md');
}

export default class TnC extends Component {
  static propTypes = {
    hasAccepted: PropTypes.bool.isRequired,
    onAccept: PropTypes.func.isRequired
  }

  render () {
    const { hasAccepted, onAccept } = this.props;

    return (
      <div className={ styles.tnc }>
        <ReactMarkdown
          className={ styles.markdown }
          source={ tnc }
        />
        <Checkbox
          className={ styles.accept }
          label={
            <FormattedMessage
              id='firstRun.tnc.accept'
              defaultMessage='I accept these terms and conditions'
            />
          }
          checked={ hasAccepted }
          onCheck={ onAccept }
        />
      </div>
    );
  }
}
