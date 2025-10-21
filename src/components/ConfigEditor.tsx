import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AppConfig, DownloadMode, CoverFormat, ConfigValidationError } from '../types';
import './ConfigEditor.css';

const ConfigEditor: React.FC = () => {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [originalConfig, setOriginalConfig] = useState<AppConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [errors, setErrors] = useState<ConfigValidationError[]>([]);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // Load configuration on component mount
  useEffect(() => {
    loadConfig();
  }, []);

  // Check for changes whenever config updates
  useEffect(() => {
    if (config && originalConfig) {
      setHasChanges(JSON.stringify(config) !== JSON.stringify(originalConfig));
    }
  }, [config, originalConfig]);

  const loadConfig = async () => {
    setIsLoading(true);
    try {
      const loadedConfig = await invoke('get_config') as AppConfig;
      setConfig(loadedConfig);
      setOriginalConfig(JSON.parse(JSON.stringify(loadedConfig))); // Deep copy
      setErrors([]);
    } catch (error) {
      console.error('Failed to load config:', error);
      setErrors([{ field: 'general', message: `Failed to load configuration: ${error}` }]);
    } finally {
      setIsLoading(false);
    }
  };

  const validateConfig = (configToValidate: AppConfig): ConfigValidationError[] => {
    const validationErrors: ConfigValidationError[] = [];

    // Validate paths
    if (!configToValidate.output_path.trim()) {
      validationErrors.push({ field: 'output_path', message: 'Output path is required' });
    }

    if (!configToValidate.temp_path.trim()) {
      validationErrors.push({ field: 'temp_path', message: 'Temp path is required' });
    }

    // Validate itag
    const validItags = ['141', '251', '140', '139', '258', '256', '327', '328'];
    if (!validItags.includes(configToValidate.itag)) {
      validationErrors.push({ field: 'itag', message: 'Invalid audio quality selection' });
    }

    // Validate concurrent limit
    if (configToValidate.concurrent_limit < 1 || configToValidate.concurrent_limit > 10) {
      validationErrors.push({ field: 'concurrent_limit', message: 'Concurrent limit must be between 1 and 10' });
    }

    // Validate cover settings
    if (configToValidate.cover_size < 100 || configToValidate.cover_size > 2000) {
      validationErrors.push({ field: 'cover_size', message: 'Cover size must be between 100 and 2000 pixels' });
    }

    if (configToValidate.cover_quality < 1 || configToValidate.cover_quality > 100) {
      validationErrors.push({ field: 'cover_quality', message: 'Cover quality must be between 1 and 100' });
    }

    // Validate templates
    if (!configToValidate.template_folder.trim()) {
      validationErrors.push({ field: 'template_folder', message: 'Folder template is required' });
    }

    if (!configToValidate.template_file.trim()) {
      validationErrors.push({ field: 'template_file', message: 'File template is required' });
    }

    // Validate truncate if provided
    if (configToValidate.truncate !== undefined && configToValidate.truncate < 1) {
      validationErrors.push({ field: 'truncate', message: 'Truncate value must be positive' });
    }

    return validationErrors;
  };

  const handleSave = async () => {
    if (!config) return;

    const validationErrors = validateConfig(config);
    if (validationErrors.length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsSaving(true);
    setErrors([]);
    setSuccessMessage(null);

    try {
      await invoke('update_config', { config });
      setOriginalConfig(JSON.parse(JSON.stringify(config))); // Update original config
      setSuccessMessage('Configuration saved successfully!');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (error) {
      setErrors([{ field: 'general', message: `Failed to save configuration: ${error}` }]);
    } finally {
      setIsSaving(false);
    }
  };

  const handleReset = () => {
    if (originalConfig) {
      setConfig(JSON.parse(JSON.stringify(originalConfig)));
      setErrors([]);
    }
  };

  const handleSelectPath = async (field: 'output_path' | 'temp_path' | 'cookies_path') => {
    try {
      const selectedPath = await invoke('select_directory') as string;
      if (selectedPath && config) {
        setConfig({ ...config, [field]: selectedPath });
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  };

  const handleSelectCookieFile = async () => {
    try {
      const selectedFile = await invoke('select_file', { 
        filters: [{ name: 'Text Files', extensions: ['txt'] }] 
      }) as string;
      if (selectedFile && config) {
        setConfig({ ...config, cookies_path: selectedFile });
      }
    } catch (error) {
      console.error('Failed to select cookie file:', error);
    }
  };

  const getFieldError = (field: string): string | undefined => {
    return errors.find(error => error.field === field)?.message;
  };

  const itagOptions = [
    { value: '141', label: 'AAC 256kbps (Best Quality)' },
    { value: '251', label: 'Opus 160kbps (High Quality)' },
    { value: '140', label: 'AAC 128kbps (Standard Quality)' },
    { value: '139', label: 'AAC 48kbps (Low Quality)' },
    { value: '258', label: 'AAC 384kbps (Premium)' },
    { value: '256', label: 'AAC 192kbps (High)' },
    { value: '327', label: 'AAC 256kbps (Alternative)' },
    { value: '328', label: 'AAC 128kbps (Alternative)' },
  ];

  if (isLoading) {
    return (
      <div className="config-editor loading">
        <div className="loading-spinner">Loading configuration...</div>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="config-editor error">
        <div className="error-message">Failed to load configuration</div>
        <button onClick={loadConfig} className="retry-button">Retry</button>
      </div>
    );
  }

  return (
    <div className="config-editor">
      <div className="config-header">
        <h2>Configuration</h2>
        <div className="config-actions">
          {hasChanges && (
            <button onClick={handleReset} className="secondary">
              Reset Changes
            </button>
          )}
          <button 
            onClick={handleSave} 
            disabled={isSaving || !hasChanges}
            className="primary"
          >
            {isSaving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Success Message */}
      {successMessage && (
        <div className="success-message">
          ✅ {successMessage}
        </div>
      )}

      {/* General Errors */}
      {errors.filter(e => e.field === 'general').map((error, index) => (
        <div key={index} className="error-message">
          ⚠️ {error.message}
        </div>
      ))}

      <div className="config-form">
        {/* Path Settings */}
        <section className="config-section">
          <h3>Path Settings</h3>
          
          <div className="form-group">
            <label htmlFor="output_path">Output Directory</label>
            <div className="path-input-group">
              <input
                id="output_path"
                type="text"
                value={config.output_path}
                onChange={(e) => setConfig({ ...config, output_path: e.target.value })}
                className={getFieldError('output_path') ? 'error' : ''}
                placeholder="Select output directory..."
              />
              <button 
                type="button" 
                onClick={() => handleSelectPath('output_path')}
                className="path-button"
              >
                Browse
              </button>
            </div>
            {getFieldError('output_path') && (
              <div className="field-error">{getFieldError('output_path')}</div>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="temp_path">Temporary Directory</label>
            <div className="path-input-group">
              <input
                id="temp_path"
                type="text"
                value={config.temp_path}
                onChange={(e) => setConfig({ ...config, temp_path: e.target.value })}
                className={getFieldError('temp_path') ? 'error' : ''}
                placeholder="Select temporary directory..."
              />
              <button 
                type="button" 
                onClick={() => handleSelectPath('temp_path')}
                className="path-button"
              >
                Browse
              </button>
            </div>
            {getFieldError('temp_path') && (
              <div className="field-error">{getFieldError('temp_path')}</div>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="cookies_path">Cookies File (Optional)</label>
            <div className="path-input-group">
              <input
                id="cookies_path"
                type="text"
                value={config.cookies_path || ''}
                onChange={(e) => setConfig({ ...config, cookies_path: e.target.value || undefined })}
                placeholder="Select cookies file..."
              />
              <button 
                type="button" 
                onClick={handleSelectCookieFile}
                className="path-button"
              >
                Browse
              </button>
            </div>
          </div>
        </section>

        {/* Download Settings */}
        <section className="config-section">
          <h3>Download Settings</h3>
          
          <div className="form-group">
            <label htmlFor="itag">Audio Quality</label>
            <select
              id="itag"
              value={config.itag}
              onChange={(e) => setConfig({ ...config, itag: e.target.value })}
              className={getFieldError('itag') ? 'error' : ''}
            >
              {itagOptions.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            {getFieldError('itag') && (
              <div className="field-error">{getFieldError('itag')}</div>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="download_mode">Download Mode</label>
            <select
              id="download_mode"
              value={config.download_mode}
              onChange={(e) => setConfig({ ...config, download_mode: e.target.value as DownloadMode })}
            >
              <option value={DownloadMode.Audio}>Audio Only</option>
              <option value={DownloadMode.Video}>Video Only</option>
              <option value={DownloadMode.AudioVideo}>Audio + Video</option>
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="concurrent_limit">Concurrent Downloads</label>
            <input
              id="concurrent_limit"
              type="number"
              min="1"
              max="10"
              value={config.concurrent_limit}
              onChange={(e) => setConfig({ ...config, concurrent_limit: parseInt(e.target.value) || 1 })}
              className={getFieldError('concurrent_limit') ? 'error' : ''}
            />
            {getFieldError('concurrent_limit') && (
              <div className="field-error">{getFieldError('concurrent_limit')}</div>
            )}
            <div className="field-help">Number of simultaneous downloads (1-10)</div>
          </div>
        </section>

        {/* Cover Art Settings */}
        <section className="config-section">
          <h3>Cover Art Settings</h3>
          
          <div className="form-group">
            <label>
              <input
                type="checkbox"
                checked={config.save_cover}
                onChange={(e) => setConfig({ ...config, save_cover: e.target.checked })}
              />
              Save cover art as separate file
            </label>
          </div>

          <div className="form-group">
            <label htmlFor="cover_size">Cover Size (pixels)</label>
            <input
              id="cover_size"
              type="number"
              min="100"
              max="2000"
              value={config.cover_size}
              onChange={(e) => setConfig({ ...config, cover_size: parseInt(e.target.value) || 500 })}
              className={getFieldError('cover_size') ? 'error' : ''}
            />
            {getFieldError('cover_size') && (
              <div className="field-error">{getFieldError('cover_size')}</div>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="cover_format">Cover Format</label>
            <select
              id="cover_format"
              value={config.cover_format}
              onChange={(e) => setConfig({ ...config, cover_format: e.target.value as CoverFormat })}
            >
              <option value={CoverFormat.Jpg}>JPEG</option>
              <option value={CoverFormat.Png}>PNG</option>
              <option value={CoverFormat.Webp}>WebP</option>
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="cover_quality">Cover Quality (%)</label>
            <input
              id="cover_quality"
              type="number"
              min="1"
              max="100"
              value={config.cover_quality}
              onChange={(e) => setConfig({ ...config, cover_quality: parseInt(e.target.value) || 90 })}
              className={getFieldError('cover_quality') ? 'error' : ''}
            />
            {getFieldError('cover_quality') && (
              <div className="field-error">{getFieldError('cover_quality')}</div>
            )}
          </div>
        </section>

        {/* File Templates */}
        <section className="config-section">
          <h3>File Templates</h3>
          
          <div className="form-group">
            <label htmlFor="template_folder">Folder Template</label>
            <input
              id="template_folder"
              type="text"
              value={config.template_folder}
              onChange={(e) => setConfig({ ...config, template_folder: e.target.value })}
              className={getFieldError('template_folder') ? 'error' : ''}
              placeholder="{artist}/{album}"
            />
            {getFieldError('template_folder') && (
              <div className="field-error">{getFieldError('template_folder')}</div>
            )}
            <div className="field-help">
              Available variables: {'{artist}'}, {'{album}'}, {'{year}'}, {'{genre}'}
            </div>
          </div>

          <div className="form-group">
            <label htmlFor="template_file">File Template</label>
            <input
              id="template_file"
              type="text"
              value={config.template_file}
              onChange={(e) => setConfig({ ...config, template_file: e.target.value })}
              className={getFieldError('template_file') ? 'error' : ''}
              placeholder="{track:02d} - {title}"
            />
            {getFieldError('template_file') && (
              <div className="field-error">{getFieldError('template_file')}</div>
            )}
            <div className="field-help">
              Available variables: {'{title}'}, {'{artist}'}, {'{track}'}, {'{album}'}
            </div>
          </div>

          <div className="form-group">
            <label htmlFor="template_date">Date Template</label>
            <input
              id="template_date"
              type="text"
              value={config.template_date}
              onChange={(e) => setConfig({ ...config, template_date: e.target.value })}
              placeholder="%Y-%m-%d"
            />
            <div className="field-help">
              Python strftime format (e.g., %Y-%m-%d for YYYY-MM-DD)
            </div>
          </div>
        </section>

        {/* Advanced Options */}
        <section className="config-section">
          <h3>Advanced Options</h3>
          
          <div className="form-group">
            <label htmlFor="po_token">PO Token (Optional)</label>
            <input
              id="po_token"
              type="password"
              value={config.po_token || ''}
              onChange={(e) => setConfig({ ...config, po_token: e.target.value || undefined })}
              placeholder="Enter PO token for premium content..."
            />
            <div className="field-help">
              Required for downloading premium content and private playlists
            </div>
          </div>

          <div className="form-group">
            <label htmlFor="exclude_tags">Exclude Tags (Optional)</label>
            <input
              id="exclude_tags"
              type="text"
              value={config.exclude_tags || ''}
              onChange={(e) => setConfig({ ...config, exclude_tags: e.target.value || undefined })}
              placeholder="tag1,tag2,tag3"
            />
            <div className="field-help">
              Comma-separated list of metadata tags to exclude
            </div>
          </div>

          <div className="form-group">
            <label htmlFor="truncate">Truncate Filenames (Optional)</label>
            <input
              id="truncate"
              type="number"
              min="1"
              value={config.truncate || ''}
              onChange={(e) => setConfig({ ...config, truncate: e.target.value ? parseInt(e.target.value) : undefined })}
              placeholder="255"
            />
            {getFieldError('truncate') && (
              <div className="field-error">{getFieldError('truncate')}</div>
            )}
            <div className="field-help">
              Maximum filename length (leave empty for no limit)
            </div>
          </div>

          <div className="form-group">
            <label>
              <input
                type="checkbox"
                checked={config.overwrite}
                onChange={(e) => setConfig({ ...config, overwrite: e.target.checked })}
              />
              Overwrite existing files
            </label>
          </div>

          <div className="form-group">
            <label>
              <input
                type="checkbox"
                checked={config.no_synced_lyrics}
                onChange={(e) => setConfig({ ...config, no_synced_lyrics: e.target.checked })}
              />
              Skip synced lyrics download
            </label>
          </div>
        </section>
      </div>
    </div>
  );
};

export default ConfigEditor;