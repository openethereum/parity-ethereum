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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { debounce } from 'lodash';
import { FormattedMessage } from 'react-intl';
import zxcvbn from 'zxcvbn';

import LabelWrapper from '../LabelWrapper';
import Progress from '../../Progress';

import styles from './passwordStrength.css';

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

    return (
      <LabelWrapper
        className={ styles.strength }
        label={
          <FormattedMessage
            id='ui.passwordStrength.label'
            defaultMessage='password strength'
          />
        }
      >
        <Progress
          color={ this.getStrengthBarColor(score) }
          isDeterminate
          max={ 100 }
          value={ score * 100 / 5 + 20 }
        />
        <div className={ styles.feedback }>
          { this.renderFeedback(feedback) }
        </div>
      </LabelWrapper>
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
        return 'green';

      case 3:
        return 'blue';

      case 2:
        return 'yellow';

      case 1:
        return 'orange';

      default:
        return 'red';
    }
  }
}
