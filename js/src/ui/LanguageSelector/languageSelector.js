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

import { MenuItem } from 'material-ui';
import { observer } from 'mobx-react';
import React, { Component } from 'react';
import { FormattedMessage } from 'react-intl';

import { LocaleStore } from '~/i18n';
import { FeaturesStore, FEATURES } from '../Features';

import Select from '../Form/Select';

@observer
export default class LanguageSelector extends Component {
  features = FeaturesStore.get();
  store = LocaleStore.get();

  render () {
    if (!this.features.active[FEATURES.LANGUAGE]) {
      return null;
    }

    return (
      <Select
        hint={
          <FormattedMessage
            id='settings.parity.languages.hint'
            defaultMessage='the language this interface is displayed with'
          />
        }
        label={
          <FormattedMessage
            id='settings.parity.languages.label'
            defaultMessage='language'
          />
        }
        value={ this.store.locale }
        onChange={ this.onChange }
      >
        { this.renderOptions() }
      </Select>
    );
  }

  renderOptions () {
    return this.store.locales.map((locale) => {
      const label = <FormattedMessage id={ `languages.${locale}` } />;

      return (
        <MenuItem
          key={ locale }
          value={ locale }
          label={ label }
        >
          { label }
        </MenuItem>
      );
    });
  }

  onChange = (event, index, locale) => {
    this.store.setLocale(locale || event.target.value);
  }
}
