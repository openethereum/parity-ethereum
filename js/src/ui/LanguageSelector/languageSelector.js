// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { MenuItem } from 'material-ui';

import Select from '../Form/Select';
import Translate from '../Translate';
import { getLocale, setLocale } from '../../i18n';

const LOCALES = [
  'en', 'de'
];

export default class LanguageSelector extends Component {
  state = {
    locale: getLocale()
  }

  render () {
    const { locale } = this.state;
    const hint = <Translate value='settings.parity.languages.hint' />;
    const label = <Translate value='settings.parity.languages.label' />;

    return (
      <Select
        hint={ hint }
        label={ label }
        value={ locale }
        onChange={ this.onChange }>
        { this.renderOptions() }
      </Select>
    );
  }

  renderOptions () {
    return LOCALES.map((locale) => {
      const label = <Translate value={ `settings.parity.languages.language_${locale}` } />;

      return (
        <MenuItem
          key={ locale }
          value={ locale }
          label={ label }>
          { label }
        </MenuItem>
      );
    });
  }

  onChange = (event, index, locale) => {
    this.setState({ locale }, () => {
      setLocale(locale);
    });
  }
}
