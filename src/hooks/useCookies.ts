import { useState, useEffect, useCallback } from 'react';
import { 
  CookieValidationResult, 
  CookieImportResult,
  BrowserType,
  BrowserCookieInstructions 
} from '../types';
import { api } from '../services/api';
import { useErrorHandler } from '../services/errorHandler';
import { loadingStateManager } from '../services/loadingState';

export interface UseCookiesReturn {
  // State
  validationResult: CookieValidationResult | null;
  isLoading: boolean;
  isImporting: boolean;
  error: string | null;
  lastImportResult: CookieImportResult | null;

  // Actions
  importCookies: (filePath: string) => Promise<void>;
  validateCookies: () => Promise<void>;
  clearCookies: () => Promise<void>;
  getBrowserInstructions: (browser: BrowserType) => BrowserCookieInstructions;
  checkCookieExpiration: () => boolean;
}

const BROWSER_INSTRUCTIONS: Record<BrowserType, BrowserCookieInstructions> = {
  [BrowserType.Chrome]: {
    browser: BrowserType.Chrome,
    steps: [
      "Install a browser extension like 'Get cookies.txt LOCALLY'",
      "Navigate to music.youtube.com and log in",
      "Click the extension icon and download cookies",
      "Save the file and import it here"
    ],
    cookie_path: "Chrome/User Data/Default/Cookies"
  },
  [BrowserType.Firefox]: {
    browser: BrowserType.Firefox,
    steps: [
      "Install the 'cookies.txt' Firefox addon",
      "Navigate to music.youtube.com and log in", 
      "Click the addon icon and export cookies",
      "Save the cookies.txt file and import it here"
    ],
    cookie_path: "Firefox/Profiles/*/cookies.sqlite"
  },
  [BrowserType.Safari]: {
    browser: BrowserType.Safari,
    steps: [
      "Use Safari's Web Inspector (Develop menu)",
      "Navigate to music.youtube.com and log in",
      "Open Web Inspector > Storage > Cookies",
      "Export cookies manually or use a third-party tool"
    ]
  },
  [BrowserType.Edge]: {
    browser: BrowserType.Edge,
    steps: [
      "Install a browser extension like 'Get cookies.txt LOCALLY'",
      "Navigate to music.youtube.com and log in",
      "Click the extension icon and download cookies",
      "Save the file and import it here"
    ],
    cookie_path: "Edge/User Data/Default/Cookies"
  },
  [BrowserType.Opera]: {
    browser: BrowserType.Opera,
    steps: [
      "Install a browser extension for cookie export",
      "Navigate to music.youtube.com and log in",
      "Export cookies using the extension",
      "Save the cookies.txt file and import it here"
    ]
  }
};

export const useCookies = (): UseCookiesReturn => {
  const [validationResult, setValidationResult] = useState<CookieValidationResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastImportResult, setLastImportResult] = useState<CookieImportResult | null>(null);
  
  const { handleApiError, handleSuccess, handleWarning } = useErrorHandler();

  // Import cookies from a file
  const importCookies = useCallback(async (filePath: string) => {
    try {
      setIsImporting(true);
      setError(null);
      loadingStateManager.setLoading('cookie-import', true);
      
      const result = await api.cookies.importCookies(filePath);
      
      setLastImportResult(result);
      
      if (!result.success) {
        throw new Error(result.error || 'Failed to import cookies');
      }
      
      handleSuccess(`Successfully imported ${result.cookies_count || 0} cookies`);
      
      // Validate cookies after successful import
      await validateCookies();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to import cookies';
      setError(errorMessage);
      handleApiError(err, 'Import Cookies');
      throw err;
    } finally {
      setIsImporting(false);
      loadingStateManager.setLoading('cookie-import', false);
    }
  }, [handleApiError, handleSuccess]);

  // Validate current cookies
  const validateCookies = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      loadingStateManager.setLoading('cookie-validate', true);
      
      const result = await api.cookies.validateCookies();
      setValidationResult(result);
      
      if (!result.is_valid && result.error) {
        setError(result.error);
        handleWarning(result.error, 'Cookie Validation');
      } else if (result.is_valid && result.days_until_expiry && result.days_until_expiry <= 7) {
        handleWarning(`Cookies will expire in ${result.days_until_expiry} days`, 'Cookie Expiration Warning');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to validate cookies';
      setError(errorMessage);
      handleApiError(err, 'Validate Cookies');
    } finally {
      setIsLoading(false);
      loadingStateManager.setLoading('cookie-validate', false);
    }
  }, [handleApiError, handleWarning]);

  // Clear current cookies
  const clearCookies = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      await api.cookies.clearCookies();
      setValidationResult(null);
      setLastImportResult(null);
      
      handleSuccess('Cookies cleared successfully');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to clear cookies';
      setError(errorMessage);
      handleApiError(err, 'Clear Cookies');
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [handleApiError, handleSuccess]);

  // Get browser-specific instructions
  const getBrowserInstructions = useCallback((browser: BrowserType): BrowserCookieInstructions => {
    return BROWSER_INSTRUCTIONS[browser];
  }, []);

  // Check if cookies are expiring soon (within 7 days)
  const checkCookieExpiration = useCallback((): boolean => {
    if (!validationResult || !validationResult.days_until_expiry) {
      return false;
    }
    
    return validationResult.days_until_expiry <= 7;
  }, [validationResult]);

  // Load initial cookie validation on mount
  useEffect(() => {
    validateCookies();
  }, [validateCookies]);

  return {
    // State
    validationResult,
    isLoading,
    isImporting,
    error,
    lastImportResult,

    // Actions
    importCookies,
    validateCookies,
    clearCookies,
    getBrowserInstructions,
    checkCookieExpiration,
  };
};