import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DownloadJob, JobStatus, DownloadStage } from '../types';
import './QueueItem.css';

interface QueueItemProps {
  job: DownloadJob;
  onJobUpdate: () => void;
}

const QueueItem: React.FC<QueueItemProps> = ({ job, onJobUpdate }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isRetrying, setIsRetrying] = useState(false);
  const [isCancelling, setIsCancelling] = useState(false);
  const [isRemoving, setIsRemoving] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);

  const getStageIcon = (stage: DownloadStage): string => {
    switch (stage) {
      case DownloadStage.Initializing:
        return '🔄';
      case DownloadStage.FetchingMetadata:
        return '📋';
      case DownloadStage.DownloadingAudio:
        return '⬇️';
      case DownloadStage.Remuxing:
        return '🔧';
      case DownloadStage.ApplyingTags:
        return '🏷️';
      case DownloadStage.Finalizing:
        return '✅';
      case DownloadStage.Completed:
        return '✅';
      case DownloadStage.Failed:
        return '❌';
      default:
        return '⏳';
    }
  };

  const getStatusIcon = (status: JobStatus): string => {
    switch (status) {
      case JobStatus.Queued:
        return '⏳';
      case JobStatus.Downloading:
        return '⬇️';
      case JobStatus.Completed:
        return '✅';
      case JobStatus.Failed:
        return '❌';
      case JobStatus.Cancelled:
        return '⏹️';
      default:
        return '❓';
    }
  };

  const handleRetry = async () => {
    setIsRetrying(true);
    try {
      await invoke('retry_job', { jobId: job.id });
      onJobUpdate();
    } catch (error) {
      console.error('Failed to retry job:', error);
    } finally {
      setIsRetrying(false);
    }
  };

  const handleCancel = async () => {
    setIsCancelling(true);
    try {
      await invoke('cancel_job', { jobId: job.id });
      onJobUpdate();
    } catch (error) {
      console.error('Failed to cancel job:', error);
    } finally {
      setIsCancelling(false);
    }
  };

  const handleRemove = async () => {
    setIsRemoving(true);
    try {
      await invoke('remove_job', { jobId: job.id });
      onJobUpdate();
    } catch (error) {
      console.error('Failed to remove job:', error);
    } finally {
      setIsRemoving(false);
    }
  };

  const handleToggleExpanded = async () => {
    if (!isExpanded && logs.length === 0) {
      // Load logs when expanding for the first time
      try {
        const jobLogs = await invoke('get_job_logs', { jobId: job.id });
        setLogs((jobLogs as string[]) || []);
      } catch (error) {
        console.error('Failed to load job logs:', error);
        setLogs(['Failed to load logs']);
      }
    }
    setIsExpanded(!isExpanded);
  };

  const formatDuration = (startTime?: string, endTime?: string): string => {
    if (!startTime) return '';
    
    const start = new Date(startTime);
    const end = endTime ? new Date(endTime) : new Date();
    const duration = Math.floor((end.getTime() - start.getTime()) / 1000);
    
    if (duration < 60) return `${duration}s`;
    if (duration < 3600) return `${Math.floor(duration / 60)}m ${duration % 60}s`;
    return `${Math.floor(duration / 3600)}h ${Math.floor((duration % 3600) / 60)}m`;
  };

  const canRetry = job.status === JobStatus.Failed || job.status === JobStatus.Cancelled;
  const canCancel = job.status === JobStatus.Queued || job.status === JobStatus.Downloading;
  const canRemove = job.status !== JobStatus.Downloading;

  return (
    <div className={`queue-item ${job.status}`}>
      <div className="queue-item-header">
        <div className="job-main-info">
          <div className="job-status-line">
            <span className="status-icon">{getStatusIcon(job.status)}</span>
            <span className={`status-text ${job.status}`}>
              {job.status.charAt(0).toUpperCase() + job.status.slice(1)}
            </span>
            {job.status === JobStatus.Downloading && job.progress && (
              <span className="stage-indicator">
                {getStageIcon(job.progress.stage)} {job.progress.stage.replace('_', ' ')}
              </span>
            )}
          </div>

          <div className="job-url-line">
            <span className="job-url">{job.url}</span>
            <button 
              className="expand-button"
              onClick={handleToggleExpanded}
              aria-label={isExpanded ? 'Collapse details' : 'Expand details'}
            >
              {isExpanded ? '▼' : '▶'}
            </button>
          </div>

          {job.metadata && (
            <div className="job-metadata">
              {job.metadata.title && (
                <div className="metadata-row">
                  <span className="metadata-label">Title:</span>
                  <span className="metadata-value title">{job.metadata.title}</span>
                </div>
              )}
              {job.metadata.artist && (
                <div className="metadata-row">
                  <span className="metadata-label">Artist:</span>
                  <span className="metadata-value artist">{job.metadata.artist}</span>
                </div>
              )}
              {job.metadata.album && (
                <div className="metadata-row">
                  <span className="metadata-label">Album:</span>
                  <span className="metadata-value album">{job.metadata.album}</span>
                </div>
              )}
              {job.metadata.duration && (
                <div className="metadata-row">
                  <span className="metadata-label">Duration:</span>
                  <span className="metadata-value duration">
                    {Math.floor(job.metadata.duration / 60)}:{(job.metadata.duration % 60).toString().padStart(2, '0')}
                  </span>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="job-actions">
          {canRetry && (
            <button 
              onClick={handleRetry}
              disabled={isRetrying}
              className="action-button retry"
              title="Retry job"
            >
              {isRetrying ? '🔄' : '🔄'} Retry
            </button>
          )}
          {canCancel && (
            <button 
              onClick={handleCancel}
              disabled={isCancelling}
              className="action-button cancel"
              title="Cancel job"
            >
              {isCancelling ? '⏳' : '⏹️'} Cancel
            </button>
          )}
          {canRemove && (
            <button 
              onClick={handleRemove}
              disabled={isRemoving}
              className="action-button remove"
              title="Remove job"
            >
              {isRemoving ? '⏳' : '🗑️'} Remove
            </button>
          )}
        </div>
      </div>

      {/* Progress Bar for Downloading Jobs */}
      {job.status === JobStatus.Downloading && job.progress && (
        <div className="progress-section">
          <div className="progress-info">
            <span className="progress-text">{job.progress.current_step}</span>
            <div className="progress-details">
              {job.progress.current_step_index && job.progress.total_steps && (
                <span className="step-counter">
                  Step {job.progress.current_step_index} of {job.progress.total_steps}
                </span>
              )}
              {job.progress.percentage !== undefined && (
                <span className="percentage">{job.progress.percentage.toFixed(1)}%</span>
              )}
            </div>
          </div>
          <div className="progress-bar">
            <div 
              className="progress-fill"
              style={{ width: `${job.progress.percentage || 0}%` }}
            />
          </div>
        </div>
      )}

      {/* Error Display */}
      {job.error && (
        <div className="job-error">
          <span className="error-icon">⚠️</span>
          <span className="error-text">{job.error}</span>
        </div>
      )}

      {/* Timing Information */}
      <div className="job-timing">
        <span className="created-time">
          Created: {new Date(job.created_at).toLocaleString()}
        </span>
        {job.started_at && (
          <span className="started-time">
            Started: {new Date(job.started_at).toLocaleString()}
          </span>
        )}
        {job.completed_at && (
          <span className="completed-time">
            Completed: {new Date(job.completed_at).toLocaleString()}
          </span>
        )}
        {job.started_at && (
          <span className="duration">
            Duration: {formatDuration(job.started_at, job.completed_at)}
          </span>
        )}
      </div>

      {/* Expandable Log Viewer */}
      {isExpanded && (
        <div className="log-viewer">
          <div className="log-header">
            <h4>Job Logs</h4>
            <button 
              className="log-refresh"
              onClick={handleToggleExpanded}
              title="Refresh logs"
            >
              🔄
            </button>
          </div>
          <div className="log-content">
            {logs.length > 0 ? (
              logs.map((log, index) => (
                <div key={index} className="log-line">
                  {log}
                </div>
              ))
            ) : (
              <div className="log-empty">No logs available</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default QueueItem;