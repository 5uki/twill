import { useState } from 'react';
import { FluentProvider } from '@fluentui/react-components';
import { twillTheme } from './theme';
import './App.css';
import { Sidebar } from './components/Sidebar';
import { TopHeader } from './components/TopHeader';
import { MailWorkspace } from './components/MailWorkspace';

import { Titlebar } from './components/Titlebar';

function App() {
  const [activeCategory, setActiveCategory] = useState('inbox');

  return (
    <FluentProvider theme={twillTheme}>
      <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', overflow: 'hidden' }}>
        <Titlebar />
        <div className="app-container" style={{ flex: 1, minHeight: 0 }}>
          <Sidebar activeCategory={activeCategory} onCategoryChange={setActiveCategory} />
          <div className="main-workspace">
            <TopHeader />
            <MailWorkspace category={activeCategory} />
          </div>
        </div>
      </div>
    </FluentProvider>
  );
}

export default App;
