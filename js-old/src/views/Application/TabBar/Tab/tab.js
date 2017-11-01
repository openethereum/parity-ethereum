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

import { Tab as MUITab } from 'material-ui/Tabs';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Badge } from '~/ui';

import styles from '../tabBar.css';

const SIGNER_ID = 'signer';

export default class Tab extends Component {
  static propTypes = {
    pendings: PropTypes.number,
    view: PropTypes.object.isRequired
  };

  render () {
    const { view } = this.props;

    return (
      <MUITab
        icon={ view.icon }
        label={
          view.id === SIGNER_ID
            ? this.renderSignerLabel()
            : this.renderLabel(view.id)
        }
      />
    );
  }

  renderLabel (id, bubble) {
    return (
      <div className={ styles.label }>
        <FormattedMessage
          id={ `settings.views.${id}.label` }
        />
        { bubble }
      </div>
    );
  }

  renderSignerLabel () {
    const { pendings } = this.props;
    let bubble;

    if (pendings) {
      bubble = (
        <Badge
          color='red'
          className={ styles.labelBubble }
          value={ pendings }
        />
      );
    }

    return this.renderLabel(SIGNER_ID, bubble);
  }
}
