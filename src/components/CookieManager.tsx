import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './CookieManager.css';

interface CookieStatus {
  isValid: boolean;
  expirationDate?: string;
  hasPoToken: boolean;
  message: string;
}

const CookieManager: React.FC = () => {
  const [cookieStatus, setCookieStatus] = useState<CookieStatus | null>(null);
  const [poToken, setPoToken] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [selectedBrowser, setSelectedBrowser] = useState<string>('chrome');
  const [showInstructions, setShowInstructions] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Load cookie status on component mount
  useEffect(() => {
    loadCookieStatus();
  }, []);

  const loadCookieStatus = async () => {
    try {
      const status = await invoke('get_cookie_status') as CookieStatus;
      setCookieStatus(status);
      
      // Load PO token if available
      const config = await invoke('get_config') as any;
      setPoToken(config.po_token || '');
    } catch (error) {
      console.error('Failed to load cookie status:', error);
      setCookieStatus({
        isValid: false,
        hasPoToken: false,
        message: 'No cookies imported'
      });
    }
  };

  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    if (!file.name.endsWith('.txt')) {
      setError('Please select a .txt file containing cookies');
      return;
    }

    setIsLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      // Read file content
      const content = await file.text();
      
      // Validate cookie format
      if (!validateCookieFormat(content)) {
        setError('Invalid cookie format. Please ensure the file is in Netscape format.');
        return;
      }

      // Import cookies
      await invoke('import_cookies', { cookieContent: content });
      setSuccessMessage('Cookies imported successfully!');
      
      // Refresh status
      await loadCookieStatus();
      
      // Clear success message after 3 seconds
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (error) {
      setError(`Failed to import cookies: ${error}`);
    } finally {
      setIsLoading(false);
      // Clear file input
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  const validateCookieFormat = (content: string): boolean => {
    const lines = content.split('\n').filter(line => line.trim() && !line.startsWith('#'));
    
    if (lines.length === 0) return false;
    
    // Check if at least one line has the expected cookie format
    return lines.some(line => {
      const parts = line.split('\t');
      return parts.length >= 6 && (
        parts[0].includes('youtube.com') || 
        parts[0].includes('music.youtube.com')
      );
    });
  };

  const handleValidateCookies = async () => {
    setIsValidating(true);
    setError(null);
    
    try {
      const isValid = await invoke('validate_cookies') as boolean;
      if (isValid) {
        setSuccessMessage('Cookies are valid and working!');
        setTimeout(() => setSuccessMessage(null), 3000);
      } else {
        setError('Cookies are invalid or expired. Please import new cookies.');
      }
      await loadCookieStatus();
    } catch (error) {
      setError(`Cookie validation failed: ${error}`);
    } finally {
      setIsValidating(false);
    }
  };

  const handleSavePoToken = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      await invoke('update_po_token', { poToken: poToken.trim() || null });
      setSuccessMessage('PO Token saved successfully!');
      await loadCookieStatus();
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (error) {
      setError(`Failed to save PO Token: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClearCookies = async () => {
    if (!confirm('Are you sure you want to clear all cookies? This will require re-importing them.')) {
      return;
    }
    
    setIsLoading(true);
    try {
      await invoke('clear_cookies');
      setSuccessMessage('Cookies cleared successfully!');
      await loadCookieStatus();
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (error) {
      setError(`Failed to clear cookies: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const getBrowserInstructions = (browser: string) => {
    const instructions = {
      chrome: {
        name: 'Google Chrome',
        steps: [
          'Install the "Get cookies.txt LOCALLY" extension from Chrome Web Store',
          'Navigate to music.youtube.com and log in',
          'Click the extension icon in the toolbar',
          'Click "Export" to download cookies.txt',
          'Import the downloaded file using the button above'
        ],
        extensionUrl: 'https://chrome.google.com/webstore/detail/get-cookiestxt-locally/cclelndahbckbenkjhflpdbgdldlbecc'
      },
      firefox: {
        name: 'Mozilla Firefox',
        steps: [
          'Install the "cookies.txt" add-on from Firefox Add-ons',
          'Navigate to music.youtube.com and log in',
          'Right-click on the page and select "cookies.txt"',
          'Save the cookies.txt file',
          'Import the saved file using the button above'
        ],
        extensionUrl: 'https://addons.mozilla.org/en-US/firefox/addon/cookies-txt/'
      },
      edge: {
        name: 'Microsoft Edge',
        steps: [
          'Install the "Get cookies.txt LOCALLY" extension from Edge Add-ons',
          'Navigate to music.youtube.com and log in',
          'Click the extension icon in the toolbar',
          'Click "Export" to download cookies.txt',
          'Import the downloaded file using the button above'
        ],
        extensionUrl: 'https://microsoftedge.microsoft.com/addons/detail/get-cookiestxt-locally/eeecpophkdmbnfcbcdemjcbnjfhokgmn'
      },
      safari: {
        name: 'Safari',
        steps: [
          'Open Safari Developer Tools (Develop > Show Web Inspector)',
          'Navigate to music.youtube.com and log in',
          'Go to the Storage tab in Developer Tools',
          'Select Cookies > music.youtube.com',
          'Manually copy cookie values or use a third-party tool',
          'Format as Netscape cookies.txt and import above'
        ],
        extensionUrl: null
      }
    };
    
    return instructions[browser as keyof typeof instructions] || instructions.chrome;
  };

  const currentInstructions = getBrowserInstructions(selectedBrowser);

  return (
    <div className="cookie-manager">
      <div className="cookie-header">
        <h2>Cookie Manager</h2>
        <p className="cookie-subtitle">
          Import YouTube Music cookies to access premium content and private playlists
        </p>
      </div>

      {/* Success Message */}
      {successMessage && (
        <div className="success-message">
          ✅ {successMessage}
        </div>
      )}

      {/* Error Message */}
      {error && (
        <div className="error-message">
          ⚠️ {error}
          <button onClick={() => setError(null)} className="error-close">×</button>
        </div>
      )}

      {/* Cookie Status */}
      <div className="cookie-status-section">
        <h3>Current Status</h3>
        <div className={`status-card ${cookieStatus?.isValid ? 'valid' : 'invalid'}`}>
          <div className="status-indicator">
            <span className="status-icon">
              {cookieStatus?.isValid ? '✅' : '❌'}
            </span>
            <span className="status-text">
              {cookieStatus?.isValid ? 'Cookies Valid' : 'No Valid Cookies'}
            </span>
          </div>
          
          <div className="status-details">
            <p>{cookieStatus?.message || 'Loading...'}</p>
            {cookieStatus?.expirationDate && (
              <p className="expiration-info">
                Expires: {new Date(cookieStatus.expirationDate).toLocaleDateString()}
              </p>
            )}
            {cookieStatus?.hasPoToken && (
              <p className="po-token-info">✅ PO Token configured</p>
            )}
          </div>

          <div className="status-actions">
            <button 
              onClick={handleValidateCookies}
              disabled={isValidating}
              className="secondary"
            >
              {isValidating ? 'Validating...' : '🔄 Test Cookies'}
            </button>
            {cookieStatus?.isValid && (
              <button 
                onClick={handleClearCookies}
                disabled={isLoading}
                className="danger"
              >
                🗑️ Clear Cookies
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Cookie Import */}
      <div className="cookie-import-section">
        <h3>Import Cookies</h3>
        <div className="import-card">
          <div className="file-drop-zone">
            <input
              ref={fileInputRef}
              type="file"
              accept=".txt"
              onChange={handleFileSelect}
              disabled={isLoading}
              className="file-input"
            />
            <div className="drop-zone-content">
              <span className="upload-icon">📁</span>
              <p className="drop-text">
                {isLoading ? 'Importing cookies...' : 'Select cookies.txt file'}
              </p>
              <p className="drop-subtext">
                Netscape format cookies exported from your browser
              </p>
              <button 
                onClick={() => fileInputRef.current?.click()}
                disabled={isLoading}
                className="upload-button"
              >
                {isLoading ? 'Importing...' : 'Choose File'}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* PO Token */}
      <div className="po-token-section">
        <h3>PO Token (Optional)</h3>
        <div className="po-token-card">
          <p className="po-token-description">
            PO Token is required for downloading premium content and accessing private playlists.
            You can extract it from your browser's network requests when using YouTube Music.
          </p>
          
          <div className="po-token-input-group">
            <input
              type="password"
              value={poToken}
              onChange={(e) => setPoToken(e.target.value)}
              placeholder="Enter PO Token (optional)"
              className="po-token-input"
            />
            <button 
              onClick={handleSavePoToken}
              disabled={isLoading}
              className="primary"
            >
              Save Token
            </button>
          </div>
          
          <div className="po-token-help">
            <details>
              <summary>How to find PO Token</summary>
              <ol>
                <li>Open browser developer tools (F12)</li>
                <li>Go to Network tab</li>
                <li>Play any song on YouTube Music</li>
                <li>Look for requests to "youtubei/v1/player"</li>
                <li>Find the "po_token" parameter in the request</li>
                <li>Copy the value and paste it above</li>
              </ol>
            </details>
          </div>
        </div>
      </div>

      {/* Browser Instructions */}
      <div className="instructions-section">
        <div className="instructions-header">
          <h3>Cookie Extraction Instructions</h3>
          <button 
            onClick={() => setShowInstructions(!showInstructions)}
            className="toggle-instructions"
          >
            {showInstructions ? '▼ Hide' : '▶ Show'} Instructions
          </button>
        </div>

        {showInstructions && (
          <div className="instructions-content">
            <div className="browser-selector">
              <label>Select your browser:</label>
              <select 
                value={selectedBrowser} 
                onChange={(e) => setSelectedBrowser(e.target.value)}
              >
                <option value="chrome">Google Chrome</option>
                <option value="firefox">Mozilla Firefox</option>
                <option value="edge">Microsoft Edge</option>
                <option value="safari">Safari</option>
              </select>
            </div>

            <div className="browser-instructions">
              <h4>{currentInstructions.name} Instructions</h4>
              <ol>
                {currentInstructions.steps.map((step, index) => (
                  <li key={index}>{step}</li>
                ))}
              </ol>
              
              {currentInstructions.extensionUrl && (
                <div className="extension-link">
                  <a 
                    href={currentInstructions.extensionUrl} 
                    target="_blank" 
                    rel="noopener noreferrer"
                    className="extension-button"
                  >
                    📦 Get Extension
                  </a>
                </div>
              )}
            </div>

            <div className="important-notes">
              <h4>⚠️ Important Notes</h4>
              <ul>
                <li>Make sure you're logged into YouTube Music before extracting cookies</li>
                <li>Cookies may expire after some time and need to be re-imported</li>
                <li>Never share your cookies with others as they contain your login information</li>
                <li>The cookies.txt file should be in Netscape format</li>
                <li>Some premium features may require both cookies and PO Token</li>
              </ul>
            </div>
          </div>
        )}
      </div>

      {/* Expiration Warning */}
      {cookieStatus?.isValid && cookieStatus.expirationDate && (
        <div className="expiration-warning">
          <div className="warning-content">
            <span className="warning-icon">⏰</span>
            <div className="warning-text">
              <p><strong>Cookie Expiration Reminder</strong></p>
              <p>
                Your cookies will expire on {new Date(cookieStatus.expirationDate).toLocaleDateString()}.
                You'll need to re-import them after this date to continue accessing premium content.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default CookieManager;