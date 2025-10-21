import { useState, useEffect, useCallback } from 'react';
import { AppConfig, ConfigValidationResult, ConfigValidationError } from '../types';
import { api } from '../services/api';
import { useErrorHandler } from '../services/errorHandler';
import { optimisticUpdates } from '../services/optimisticUpdates';
import { loadingStateManager } from '../services/loadingState';

export interface UseConfigReturn {
  // State
  config: AppConfig | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  validationErrors: ConfigValidationError[];
  hasUnsavedChanges: boolean;

  // Actions
  updateConfig: (newConfig: Partial<AppConfig>) => void;
  saveConfig: () => Promise<void>;
  resetConfig: () => Promise<void>;
  validateConfig: (configToValidate?: AppConfig) => Promise<ConfigValidationResult>;
  loadConfig: () => Promise<void>;
}

export const useConfig = (): UseConfigReturn => {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [originalConfig, setOriginalConfig] = useState<AppConfig | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<ConfigValidationError[]>([]);
  
  const { handleApiError, handleSuccess } = useErrorHandler();

  // Check if there are unsaved changes
  const hasUnsavedChanges = config && originalConfig 
    ? JSON.stringify(config) !== JSON.stringify(originalConfig)
    : false;

  // Load configuration from backend
  const loadConfig = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      loadingStateManager.setLoading('config-load', true);
      
      const loadedConfig = await api.config.getConfig();
      setConfig(loadedConfig);
      setOriginalConfig(JSON.parse(JSON.stringify(loadedConfig))); // Deep copy
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load configuration';
      setError(errorMessage);
      handleApiError(err, 'Load Configuration');
    } finally {
      setIsLoading(false);
      loadingStateManager.setLoading('config-load', false);
    }
  }, [handleApiError]);

  // Update configuration locally (without saving)
  const updateConfig = useCallback((newConfig: Partial<AppConfig>) => {
    if (!config) return;
    
    const updatedConfig = {
      ...config,
      ...newConfig,
    };
    
    setConfig(updatedConfig);
    
    // Apply optimistic update for immediate UI feedback
    optimisticUpdates.updateConfig(newConfig);
    
    // Clear validation errors when config changes
    setValidationErrors([]);
    setError(null);
  }, [config]);

  // Validate configuration
  const validateConfig = useCallback(async (configToValidate?: AppConfig): Promise<ConfigValidationResult> => {
    try {
      const targetConfig = configToValidate || config;
      if (!targetConfig) {
        throw new Error('No configuration to validate');
      }

      const result = await api.config.validateConfig(targetConfig);
      setValidationErrors(result.errors);
      
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to validate configuration';
      setError(errorMessage);
      handleApiError(err, 'Validate Configuration');
      
      return {
        isValid: false,
        errors: [{ field: 'general', message: errorMessage }],
      };
    }
  }, [config, handleApiError]);

  // Save configuration to backend
  const saveConfig = useCallback(async () => {
    if (!config) {
      throw new Error('No configuration to save');
    }

    try {
      setIsSaving(true);
      setError(null);
      loadingStateManager.setLoading('config-save', true);

      // Validate before saving
      const validationResult = await validateConfig(config);
      if (!validationResult.isValid) {
        throw new Error('Configuration validation failed');
      }

      await api.config.updateConfig(config);
      
      // Update original config to reflect saved state
      setOriginalConfig(JSON.parse(JSON.stringify(config)));
      setValidationErrors([]);
      
      // Remove optimistic update
      optimisticUpdates.removeUpdate('config-update');
      
      handleSuccess('Configuration saved successfully');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to save configuration';
      setError(errorMessage);
      handleApiError(err, 'Save Configuration');
      throw err;
    } finally {
      setIsSaving(false);
      loadingStateManager.setLoading('config-save', false);
    }
  }, [config, validateConfig, handleApiError, handleSuccess]);

  // Reset configuration to last saved state
  const resetConfig = useCallback(async () => {
    if (originalConfig) {
      setConfig(JSON.parse(JSON.stringify(originalConfig)));
      setValidationErrors([]);
      setError(null);
    } else {
      // If no original config, reload from backend
      await loadConfig();
    }
  }, [originalConfig, loadConfig]);

  // Load initial configuration on mount
  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  return {
    // State
    config,
    isLoading,
    isSaving,
    error,
    validationErrors,
    hasUnsavedChanges,

    // Actions
    updateConfig,
    saveConfig,
    resetConfig,
    validateConfig,
    loadConfig,
  };
};