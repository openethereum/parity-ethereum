// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { parse } from 'url';
import { withRouter } from 'react-router';

import Button from '~/ui/Button';
import { LinkIcon } from '~/ui/Icons';
import Input from '~/ui/Form/Input';

// import styles from './urlButton.css';

class UrlButton extends Component {
  static propTypes = {
    router: PropTypes.object.isRequired // injected by withRouter
  };

  state = {
    inputShown: false,
    urlIsValid: false
  };

  render () {
    const { inputShown } = this.state;

    return (
      <div>
        { inputShown ? this.renderInput() : null }
        <Button
          icon={ <LinkIcon /> }
          label={
            <FormattedMessage
              id='dapps.button.url.label'
              defaultMessage='URL' />
          }
          onClick={ this.toggleInput }
        />
      </div>
    );
  }

  renderInput () {
    const { urlIsValid } = this.state;

    return (
      <Input
        hint={
          <FormattedMessage
            id='dapps.button.url.input'
            defaultMessage='https://mkr.market' />
        }
        error={ urlIsValid ? null : (
          <FormattedMessage
            id='dapps.button.url.invalid'
            defaultMessage='invalid url' />
        ) }
        onBlur={ this.hideInput }
        onChange={ this.inputOnChange }
        onFocus={ this.showInput }
        onSubmit={ this.inputOnSubmit }
        style={ { display: 'inline-block', width: '20em' } }
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

  inputOnChange = (event) => {
    const url = event.target.value;
    this.setState({
      urlIsValid: url && !!parse(url).host
    });
  }
}

export default withRouter(UrlButton);
