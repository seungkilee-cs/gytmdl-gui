import { useState } from "react";
import "./App.css";
import { QueueView, ConfigEditor, CookieManager, NotificationCenter, GlobalLoadingOverlay } from "./components";
import DebugPanel from "./components/DebugPanel";

type ActiveView = 'queue' | 'config' | 'cookies';

function App() {
  const [activeView, setActiveView] = useState<ActiveView>('queue');
  const [debugPanelVisible, setDebugPanelVisible] = useState(false);

  const renderActiveView = () => {
    switch (activeView) {
      case 'queue':
        return <QueueView />;
      case 'config':
        return <ConfigEditor />;
      case 'cookies':
        return <CookieManager />;
      default:
        return <QueueView />;
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-content">
          <h1 className="app-title">gytmdl GUI</h1>
          <p className="app-subtitle">YouTube Music Downloader</p>
        </div>
      </header>

      <nav className="app-nav">
        <button
          className={`nav-button ${activeView === 'queue' ? 'active' : ''}`}
          onClick={() => setActiveView('queue')}
        >
          <span className="nav-icon">📥</span>
          Queue
        </button>
        <button
          className={`nav-button ${activeView === 'config' ? 'active' : ''}`}
          onClick={() => setActiveView('config')}
        >
          <span className="nav-icon">⚙️</span>
          Config
        </button>
        <button
          className={`nav-button ${activeView === 'cookies' ? 'active' : ''}`}
          onClick={() => setActiveView('cookies')}
        >
          <span className="nav-icon">🍪</span>
          Cookies
        </button>
      </nav>

      <main className="app-main">
        <div className="view-container">
          {renderActiveView()}
        </div>
      </main>

      {/* Global components */}
      <NotificationCenter />
      <GlobalLoadingOverlay />
      
      {/* Debug Panel - only in development or when enabled */}
      <DebugPanel 
        isVisible={debugPanelVisible}
        onToggle={() => setDebugPanelVisible(!debugPanelVisible)}
      />
    </div>
  );
}

export default App;
