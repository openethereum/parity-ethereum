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
import ReactMarkdown from 'react-markdown';

import { SectionList } from '~/ui';

import { createRenderers } from './renderers';
import Store from './store';
import styles from './news.css';

const VERSION_ID = '1';

@observer
export default class News extends Component {
  store = Store.get();

  componentWillMount () {
    return this.store.retrieveNews(VERSION_ID);
  }

  render () {
    const { newsItems } = this.store;

    if (!newsItems || !newsItems.length) {
      return null;
    }

    return (
      <SectionList
        className={ styles.news }
        items={ newsItems }
        renderItem={ this.renderItem }
      />
    );
  }

  renderItem = (item) => {
    if (!item) {
      return null;
    }

    const inlineStyles = item.style || {};

    return (
      <div className={ styles.item }>
        <div
          className={ styles.background }
          style={ {
            backgroundImage: `url(${item.background})`
          } }
        />
        <div
          className={ styles.title }
          style={ inlineStyles.head }
        >
          { item.title }
        </div>
        <div
          className={ styles.overlay }
          style={ inlineStyles.body }
        >
          <ReactMarkdown
            className={ styles.markdown }
            renderers={ createRenderers(inlineStyles.tags) }
            source={ item.markdown }
            softBreak='br'
          />
        </div>
      </div>
    );
  }
}

export {
  VERSION_ID
};
