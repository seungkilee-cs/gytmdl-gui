import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface LogEntry {
  timestamp: string;
  level: 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';
  component: string;
  message: string;
  data?: any;
}

interface DebugPanelProps {
  isVisible: boolean;
  onToggle: () => void;
}

const DebugPanel: React.FC<DebugPanelProps> = ({ isVisible, onToggle }) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const [filter, setFilter] = useState<string>('');
  const [levelFilter, setLevelFilter] = useState<string>('ALL');

  useEffect(() => {
    if (isVisible) {
      // Start log polling
      const interval = setInterval(fetchLogs, 1000);
      return () => clearInterval(interval);
    }
  }, [isVisible]);

  const fetchLogs = async () => {
    try {
      const newLogs = await invoke<LogEntry[]>('get_debug_logs');
      setLogs(newLogs);
      
      if (autoScroll) {
        setTimeout(() => {
          const logContainer = document.getElementById('debug-log-container');
          if (logContainer) {
            logContainer.scrollTop = logContainer.scrollHeight;
          }
        }, 100);
      }
    } catch (error) {
      console.error('Failed to fetch debug logs:', error);
    }
  };

  const clearLogs = async () => {
    try {
      await invoke('clear_debug_logs');
      setLogs([]);
    } catch (error) {
      console.error('Failed to clear logs:', error);
    }
  };

  const testSidecar = async () => {
    try {
      const result = await invoke<string>('test_sidecar_binary');
      addLocalLog('INFO', 'DebugPanel', 'Sidecar test result', result);
    } catch (error) {
      addLocalLog('ERROR', 'DebugPanel', 'Sidecar test failed', error);
    }
  };

  const testDownload = async () => {
    try {
      const result = await invoke<string>('test_download_dry_run', {
        url: 'https://music.youtube.com/watch?v=dQw4w9WgXcQ'
      });
      addLocalLog('INFO', 'DebugPanel', 'Download test result', result);
    } catch (error) {
      addLocalLog('ERROR', 'DebugPanel', 'Download test failed', error);
    }
  };

  const addLocalLog = (level: LogEntry['level'], component: string, message: string, data?: any) => {
    const newLog: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      component,
      message,
      data
    };
    setLogs(prev => [...prev, newLog]);
  };

  const filteredLogs = logs.filter(log => {
    const matchesText = !filter || 
      log.message.toLowerCase().includes(filter.toLowerCase()) ||
      log.component.toLowerCase().includes(filter.toLowerCase());
    
    const matchesLevel = levelFilter === 'ALL' || log.level === levelFilter;
    
    return matchesText && matchesLevel;
  });

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'ERROR': return '#ff4444';
      case 'WARN': return '#ffaa00';
      case 'INFO': return '#4444ff';
      case 'DEBUG': return '#888888';
      default: return '#000000';
    }
  };

  if (!isVisible) {
    return (
      <button
        onClick={onToggle}
        style={{
          position: 'fixed',
          bottom: '20px',
          right: '20px',
          backgroundColor: '#333',
          color: 'white',
          border: 'none',
          borderRadius: '50%',
          width: '50px',
          height: '50px',
          cursor: 'pointer',
          fontSize: '20px',
          zIndex: 1000
        }}
        title="Open Debug Panel"
      >
        🐛
      </button>
    );
  }

  return (
    <div style={{
      position: 'fixed',
      bottom: '0',
      right: '0',
      width: '60%',
      height: '50%',
      backgroundColor: '#1a1a1a',
      color: '#ffffff',
      border: '2px solid #333',
      borderRadius: '8px 0 0 0',
      display: 'flex',
      flexDirection: 'column',
      zIndex: 1000,
      fontFamily: 'monospace',
      fontSize: '12px'
    }}>
      {/* Header */}
      <div style={{
        padding: '10px',
        backgroundColor: '#333',
        borderBottom: '1px solid #555',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center'
      }}>
        <h3 style={{ margin: 0, color: '#fff' }}>🐛 Debug Panel</h3>
        <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
          <button onClick={testSidecar} style={{ padding: '5px 10px', fontSize: '11px' }}>
            Test Sidecar
          </button>
          <button onClick={testDownload} style={{ padding: '5px 10px', fontSize: '11px' }}>
            Test Download
          </button>
          <button onClick={clearLogs} style={{ padding: '5px 10px', fontSize: '11px' }}>
            Clear
          </button>
          <button onClick={onToggle} style={{ padding: '5px 10px', fontSize: '11px' }}>
            ✕
          </button>
        </div>
      </div>

      {/* Controls */}
      <div style={{
        padding: '10px',
        backgroundColor: '#2a2a2a',
        borderBottom: '1px solid #555',
        display: 'flex',
        gap: '10px',
        alignItems: 'center'
      }}>
        <input
          type="text"
          placeholder="Filter logs..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          style={{
            padding: '5px',
            backgroundColor: '#444',
            color: '#fff',
            border: '1px solid #666',
            borderRadius: '4px',
            fontSize: '11px'
          }}
        />
        <select
          value={levelFilter}
          onChange={(e) => setLevelFilter(e.target.value)}
          style={{
            padding: '5px',
            backgroundColor: '#444',
            color: '#fff',
            border: '1px solid #666',
            borderRadius: '4px',
            fontSize: '11px'
          }}
        >
          <option value="ALL">All Levels</option>
          <option value="DEBUG">Debug</option>
          <option value="INFO">Info</option>
          <option value="WARN">Warning</option>
          <option value="ERROR">Error</option>
        </select>
        <label style={{ display: 'flex', alignItems: 'center', gap: '5px', fontSize: '11px' }}>
          <input
            type="checkbox"
            checked={autoScroll}
            onChange={(e) => setAutoScroll(e.target.checked)}
          />
          Auto-scroll
        </label>
        <span style={{ fontSize: '11px', color: '#aaa' }}>
          {filteredLogs.length} / {logs.length} logs
        </span>
      </div>

      {/* Log Display */}
      <div
        id="debug-log-container"
        style={{
          flex: 1,
          overflow: 'auto',
          padding: '10px',
          backgroundColor: '#1a1a1a'
        }}
      >
        {filteredLogs.length === 0 ? (
          <div style={{ color: '#666', fontStyle: 'italic' }}>
            No logs available. Try performing an action or click "Test Sidecar" to generate logs.
          </div>
        ) : (
          filteredLogs.map((log, index) => (
            <div
              key={index}
              style={{
                marginBottom: '5px',
                padding: '5px',
                backgroundColor: log.level === 'ERROR' ? '#331111' : 
                                log.level === 'WARN' ? '#332211' : 'transparent',
                borderLeft: `3px solid ${getLevelColor(log.level)}`,
                paddingLeft: '8px'
              }}
            >
              <div style={{ display: 'flex', gap: '10px', alignItems: 'flex-start' }}>
                <span style={{ color: '#666', minWidth: '60px' }}>
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <span
                  style={{
                    color: getLevelColor(log.level),
                    fontWeight: 'bold',
                    minWidth: '50px'
                  }}
                >
                  {log.level}
                </span>
                <span style={{ color: '#aaa', minWidth: '100px' }}>
                  {log.component}
                </span>
                <span style={{ flex: 1 }}>
                  {log.message}
                </span>
              </div>
              {log.data && (
                <div style={{
                  marginTop: '5px',
                  marginLeft: '180px',
                  color: '#ccc',
                  fontSize: '11px',
                  backgroundColor: '#333',
                  padding: '5px',
                  borderRadius: '3px',
                  whiteSpace: 'pre-wrap'
                }}>
                  {typeof log.data === 'string' ? log.data : JSON.stringify(log.data, null, 2)}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default DebugPanel;