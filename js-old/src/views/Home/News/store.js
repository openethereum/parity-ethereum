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

import { action, observable } from 'mobx';
import Contracts from '~/contracts';

let instance = null;

export default class Store {
  @observable newsItems = null;

  @action setNewsItems = (newsItems) => {
    this.newsItems = newsItems;
  }

  retrieveNews (versionId) {
    const contracts = Contracts.get();

    return contracts.registry
      .lookupMeta('paritynews', 'CONTENT')
      .then((contentId) => {
        return contracts.githubHint.getEntry(contentId);
      })
      .then(([url, owner, commit]) => {
        if (!url) {
          return null;
        }

        return fetch(url).then((response) => {
          if (!response.ok) {
            return null;
          }

          return response.json();
        });
      })
      .then((news) => {
        if (news && news[versionId]) {
          this.setNewsItems(news[versionId].items);
        }
      })
      .catch((error) => {
        console.warn('retrieveNews', error);
      });
  }

  static get () {
    if (!instance) {
      instance = new Store();
    }

    return instance;
  }
}
