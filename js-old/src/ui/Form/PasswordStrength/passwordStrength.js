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
import { debounce } from 'lodash';
import { LinearProgress } from 'material-ui';
import { FormattedMessage } from 'react-intl';
import zxcvbn from 'zxcvbn';

import styles from './passwordStrength.css';

const BAR_STYLE = {
  borderRadius: 1,
  height: 7,
  marginTop: '0.5em'
};

export default class PasswordStrength extends Component {
  static propTypes = {
    input: PropTypes.string.isRequired
  };

  state = {
    strength: null
  };

  constructor (props) {
    super(props);

    this.updateStrength = debounce(this._updateStrength, 50, { leading: true });
  }

  componentWillMount () {
    this.updateStrength(this.props.input);
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.input !== this.props.input) {
      this.updateStrength(nextProps.input);
    }
  }

  _updateStrength (input = '') {
    const strength = zxcvbn(input);

    this.setState({ strength });
  }

  render () {
    const { strength } = this.state;

    if (!strength) {
      return null;
    }

    const { score, feedback } = strength;

    // Score is between 0 and 4
    const value = score * 100 / 5 + 20;
    const color = this.getStrengthBarColor(score);

    return (
      <div className={ styles.strength }>
        <label className={ styles.label }>
          <FormattedMessage
            id='ui.passwordStrength.label'
            defaultMessage='password strength'
          />
        </label>
        <LinearProgress
          color={ color }
          mode='determinate'
          style={ BAR_STYLE }
          value={ value }
        />
        <div className={ styles.feedback }>
          { this.renderFeedback(feedback) }
        </div>
      </div>
    );
  }

  // Note that the suggestions are in english, thus it wouldn't
  // make sense to add translations to surrounding words
  renderFeedback (feedback = {}) {
    const { suggestions = [] } = feedback;

    return (
      <div>
        <p>
          { suggestions.join(' ') }
        </p>
      </div>
    );
  }

  getStrengthBarColor (score) {
    switch (score) {
      case 4:
      case 3:
        return 'lightgreen';

      case 2:
        return 'yellow';

      case 1:
        return 'orange';

      default:
        return 'red';
    }
  }
}
