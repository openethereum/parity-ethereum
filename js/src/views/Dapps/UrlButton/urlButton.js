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
import { withRouter } from 'react-router';

import Button from '~/ui/Button';
import { LinkIcon } from '~/ui/Icons';
import Input from '~/ui/Form/Input';

import styles from './urlButton.css';

const INPUT_STYLE = { display: 'inline-block', width: '20em' };

class UrlButton extends Component {
  static propTypes = {
    router: PropTypes.object.isRequired // injected by withRouter
  };

  state = {
    inputShown: false
  };

  render () {
    const { inputShown } = this.state;

    return (
      <div>
        { inputShown ? this.renderInput() : null }
        <Button
          className={ styles.button }
          icon={ <LinkIcon /> }
          label={
            <FormattedMessage
              id='dapps.button.url.label'
              defaultMessage='URL'
            />
          }
          onClick={ this.toggleInput }
        />
      </div>
    );
  }

  renderInput () {
    return (
      <Input
        hint={
          <FormattedMessage
            id='dapps.button.url.input'
            defaultMessage='https://mkr.market'
          />
        }
        onBlur={ this.hideInput }
        onFocus={ this.showInput }
        onSubmit={ this.inputOnSubmit }
        style={ INPUT_STYLE }
      />
    );
  }

  toggleInput = () => {
    const { inputShown } = this.state;

    this.setState({
      inputShown: !inputShown
    });
  }

  hideInput = () => {
    this.setState({ inputShown: false });
  }

  showInput = () => {
    this.setState({ inputShown: true });
  }

  inputOnSubmit = (url) => {
    const { router } = this.props;

    router.push(`/web/${encodeURIComponent(url)}`);
  }
}

export default withRouter(UrlButton);
