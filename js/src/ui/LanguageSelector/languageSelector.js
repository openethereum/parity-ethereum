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

import { observer } from 'mobx-react';
import React, { Component } from 'react';
import { FormattedMessage } from 'react-intl';

import { LocaleStore } from '~/i18n';
import { FeaturesStore, FEATURES } from '../Features';

import Dropdown from '../Form/Dropdown';

@observer
export default class LanguageSelector extends Component {
  features = FeaturesStore.get();
  store = LocaleStore.get();

  render () {
    if (!this.features.active[FEATURES.LANGUAGE]) {
      return null;
    }

    return (
      <Dropdown
        hint={
          <FormattedMessage
            id='settings.parity.languages.hint'
            defaultMessage='the language this interface is displayed with'
          />
        }
        label={
          <FormattedMessage
            id='settings.parity.languages.label'
            defaultMessage='UI language'
          />
        }
        value={ this.store.locale }
        onChange={ this.onChange }
        options={
          this.store.locales.map((locale) => {
            return {
              key: locale,
              value: locale,
              text: locale,
              content: <FormattedMessage id={ `languages.${locale}` } />
            };
          })
        }
      />
    );
  }

  onChange = (event, locale) => {
    this.store.setLocale(locale);
  }
}
