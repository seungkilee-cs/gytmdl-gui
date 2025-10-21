import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DownloadJob, JobStatus, QueueStats } from '../types';
import QueueItem from './QueueItem';
import './QueueView.css';

const QueueView: React.FC = () => {
  const [jobs, setJobs] = useState<DownloadJob[]>([]);
  const [urlInput, setUrlInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<JobStatus | 'all'>('all');
  const [sortBy, setSortBy] = useState<'created_at' | 'status'>('created_at');
  const [dragOver, setDragOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Calculate queue statistics
  const stats: QueueStats = {
    total: jobs.length,
    queued: jobs.filter(job => job.status === JobStatus.Queued).length,
    downloading: jobs.filter(job => job.status === JobStatus.Downloading).length,
    completed: jobs.filter(job => job.status === JobStatus.Completed).length,
    failed: jobs.filter(job => job.status === JobStatus.Failed).length,
    cancelled: jobs.filter(job => job.status === JobStatus.Cancelled).length,
  };

  // Load queue on component mount
  useEffect(() => {
    loadQueue();
    // Set up polling for real-time updates
    const interval = setInterval(loadQueue, 2000);
    return () => clearInterval(interval);
  }, []);

  const loadQueue = async () => {
    try {
      const queueData = await invoke('get_queue');
      setJobs((queueData as any).jobs || []);
      setIsPaused((queueData as any).is_paused || false);
    } catch (err) {
      console.error('Failed to load queue:', err);
    }
  };

  const validateUrl = (url: string): boolean => {
    const ytMusicPatterns = [
      /^https?:\/\/(music\.youtube\.com|www\.youtube\.com)/,
      /youtube\.com\/watch\?v=/,
      /youtube\.com\/playlist\?list=/,
      /music\.youtube\.com\/watch\?v=/,
      /music\.youtube\.com\/playlist\?list=/,
      /music\.youtube\.com\/channel\//,
      /music\.youtube\.com\/browse\//
    ];
    return ytMusicPatterns.some(pattern => pattern.test(url));
  };

  const handleAddUrl = async (url: string) => {
    if (!url.trim()) return;
    
    if (!validateUrl(url)) {
      setError('Please enter a valid YouTube Music URL');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      await invoke('add_to_queue', { url: url.trim() });
      setUrlInput('');
      await loadQueue();
    } catch (err) {
      setError(`Failed to add URL: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    handleAddUrl(urlInput);
  };

  const handlePauseResume = async () => {
    try {
      if (isPaused) {
        await invoke('resume_queue');
      } else {
        await invoke('pause_queue');
      }
      await loadQueue();
    } catch (err) {
      setError(`Failed to ${isPaused ? 'resume' : 'pause'} queue: ${err}`);
    }
  };

  const handleClearCompleted = async () => {
    try {
      const completedJobs = jobs.filter(job => 
        job.status === JobStatus.Completed || 
        job.status === JobStatus.Failed ||
        job.status === JobStatus.Cancelled
      );
      
      for (const job of completedJobs) {
        await invoke('remove_job', { jobId: job.id });
      }
      
      await loadQueue();
    } catch (err) {
      setError(`Failed to clear completed jobs: ${err}`);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    
    const text = e.dataTransfer.getData('text');
    if (text) {
      handleAddUrl(text);
    }
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file && file.type === 'text/plain') {
      const reader = new FileReader();
      reader.onload = (event) => {
        const content = event.target?.result as string;
        const urls = content.split('\n').filter(line => line.trim());
        urls.forEach(url => handleAddUrl(url));
      };
      reader.readAsText(file);
    }
  };

  // Filter and sort jobs
  const filteredJobs = jobs
    .filter(job => filter === 'all' || job.status === filter)
    .sort((a, b) => {
      if (sortBy === 'created_at') {
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      } else {
        return a.status.localeCompare(b.status);
      }
    });

  return (
    <div className="queue-view">
      <div className="queue-header">
        <h2>Download Queue</h2>
        
        {/* Queue Statistics */}
        <div className="queue-stats">
          <div className="stat-item">
            <span className="stat-value">{stats.total}</span>
            <span className="stat-label">Total</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stats.queued}</span>
            <span className="stat-label">Queued</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stats.downloading}</span>
            <span className="stat-label">Downloading</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stats.completed}</span>
            <span className="stat-label">Completed</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stats.failed}</span>
            <span className="stat-label">Failed</span>
          </div>
        </div>
      </div>

      {/* URL Input Section */}
      <div className="url-input-section">
        <form onSubmit={handleSubmit} className="url-form">
          <div 
            className={`url-input-container ${dragOver ? 'drag-over' : ''}`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <input
              type="text"
              value={urlInput}
              onChange={(e) => setUrlInput(e.target.value)}
              placeholder="Enter YouTube Music URL or drag & drop here..."
              className="url-input"
              disabled={isLoading}
            />
            <button 
              type="submit" 
              disabled={isLoading || !urlInput.trim()}
              className="add-button"
            >
              {isLoading ? 'Adding...' : 'Add'}
            </button>
          </div>
        </form>
        
        <div className="input-actions">
          <button 
            onClick={() => fileInputRef.current?.click()}
            className="secondary"
          >
            Import URLs from File
          </button>
          <input
            ref={fileInputRef}
            type="file"
            accept=".txt"
            onChange={handleFileSelect}
            style={{ display: 'none' }}
          />
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="error-message">
          {error}
          <button onClick={() => setError(null)} className="error-close">×</button>
        </div>
      )}

      {/* Queue Controls */}
      <div className="queue-controls">
        <div className="control-group">
          <button 
            onClick={handlePauseResume}
            className={isPaused ? 'primary' : 'secondary'}
          >
            {isPaused ? '▶️ Resume' : '⏸️ Pause'} Queue
          </button>
          <button 
            onClick={handleClearCompleted}
            className="secondary"
            disabled={stats.completed + stats.failed + stats.cancelled === 0}
          >
            🗑️ Clear Completed
          </button>
        </div>

        <div className="filter-sort-controls">
          <select 
            value={filter} 
            onChange={(e) => setFilter(e.target.value as JobStatus | 'all')}
            className="filter-select"
          >
            <option value="all">All Jobs</option>
            <option value={JobStatus.Queued}>Queued</option>
            <option value={JobStatus.Downloading}>Downloading</option>
            <option value={JobStatus.Completed}>Completed</option>
            <option value={JobStatus.Failed}>Failed</option>
            <option value={JobStatus.Cancelled}>Cancelled</option>
          </select>

          <select 
            value={sortBy} 
            onChange={(e) => setSortBy(e.target.value as 'created_at' | 'status')}
            className="sort-select"
          >
            <option value="created_at">Sort by Date</option>
            <option value="status">Sort by Status</option>
          </select>
        </div>
      </div>

      {/* Job List */}
      <div className="job-list">
        {filteredJobs.length === 0 ? (
          <div className="empty-state">
            <p>No jobs in queue</p>
            <p className="empty-subtitle">Add a YouTube Music URL to get started</p>
          </div>
        ) : (
          filteredJobs.map(job => (
            <QueueItem 
              key={job.id} 
              job={job} 
              onJobUpdate={loadQueue}
            />
          ))
        )}
      </div>
    </div>
  );
};

export default QueueView;