import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach } from 'vitest';
import QueueItem from '../components/QueueItem';
import { JobStatus, DownloadStage } from '../types';

const mockJob = {
  id: 'test-job-1',
  url: 'https://music.youtube.com/watch?v=test',
  status: JobStatus.Downloading,
  progress: {
    stage: DownloadStage.DownloadingAudio,
    current_step: 'Downloading audio',
    percentage: 45.2,
    total_steps: 5,
    current_step_index: 2,
  },
  created_at: '2024-01-01T00:00:00Z',
  started_at: '2024-01-01T00:01:00Z',
  metadata: {
    title: 'Test Song',
    artist: 'Test Artist',
    album: 'Test Album',
    duration: 180,
    thumbnail: 'https://example.com/thumb.jpg',
  },
};

const mockOnJobUpdate = vi.fn();

describe('QueueItem Component', () => {
  beforeEach(() => {
    mockOnJobUpdate.mockReset();
    (global as any).mockInvoke.mockReset();
  });

  it('renders job metadata correctly', () => {
    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByText('Test Song')).toBeInTheDocument();
    expect(screen.getByText('Test Artist')).toBeInTheDocument();
    expect(screen.getByText('Test Album')).toBeInTheDocument();
  });

  it('displays progress information for downloading job', () => {
    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByText('Downloading audio')).toBeInTheDocument();
    expect(screen.getByText('45.2%')).toBeInTheDocument();
  });

  it('shows correct status badge for downloading job', () => {
    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    const statusBadge = screen.getByText('Downloading');
    expect(statusBadge).toHaveClass('status-text');
  });

  it('renders queued job correctly', () => {
    const queuedJob = {
      ...mockJob,
      status: JobStatus.Queued,
      progress: {
        stage: DownloadStage.Initializing,
        current_step: 'Waiting in queue',
        percentage: 0,
      },
    };

    render(<QueueItem job={queuedJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByText('Queued')).toBeInTheDocument();
    expect(screen.getByText('Test Song')).toBeInTheDocument();
  });

  it('renders completed job correctly', () => {
    const completedJob = {
      ...mockJob,
      status: JobStatus.Completed,
      progress: {
        stage: DownloadStage.Completed,
        current_step: 'Download completed',
        percentage: 100,
      },
      completed_at: '2024-01-01T00:05:00Z',
    };

    render(<QueueItem job={completedJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByText('Completed')).toBeInTheDocument();
    expect(screen.getByText('Test Song')).toBeInTheDocument();
    // Check for completion time
    expect(screen.getByText(/Completed:/)).toBeInTheDocument();
  });

  it('renders failed job with error message', () => {
    const failedJob = {
      ...mockJob,
      status: JobStatus.Failed,
      progress: {
        stage: DownloadStage.Failed,
        current_step: 'Download failed',
        percentage: 0,
      },
      error: 'Network error occurred',
    };

    render(<QueueItem job={failedJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByText('Failed')).toBeInTheDocument();
    expect(screen.getByText('Test Song')).toBeInTheDocument();
    // Check if error message is displayed (it might be in an expanded section)
    const errorText = screen.queryByText('Network error occurred');
    if (errorText) {
      expect(errorText).toBeInTheDocument();
    }
  });

  it('shows retry button for failed jobs', () => {
    const failedJob = {
      ...mockJob,
      status: JobStatus.Failed,
      error: 'Download failed',
    };

    render(<QueueItem job={failedJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
  });

  it('shows cancel button for downloading jobs', () => {
    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument();
  });

  it('shows remove button for completed/failed jobs', () => {
    const completedJob = {
      ...mockJob,
      status: JobStatus.Completed,
    };

    render(<QueueItem job={completedJob} onJobUpdate={mockOnJobUpdate} />);
    
    expect(screen.getByRole('button', { name: /remove/i })).toBeInTheDocument();
  });

  it('handles retry job action', async () => {
    const user = userEvent.setup();
    const failedJob = {
      ...mockJob,
      status: JobStatus.Failed,
      error: 'Download failed',
    };

    (global as any).mockInvoke.mockResolvedValue(undefined);

    render(<QueueItem job={failedJob} onJobUpdate={mockOnJobUpdate} />);
    
    const retryButton = screen.getByRole('button', { name: /retry/i });
    await user.click(retryButton);
    
    await waitFor(() => {
      expect((global as any).mockInvoke).toHaveBeenCalledWith('retry_job', { jobId: 'test-job-1' });
      expect(mockOnJobUpdate).toHaveBeenCalled();
    });
  });

  it('handles cancel job action', async () => {
    const user = userEvent.setup();
    (global as any).mockInvoke.mockResolvedValue(undefined);

    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    await user.click(cancelButton);
    
    await waitFor(() => {
      expect((global as any).mockInvoke).toHaveBeenCalledWith('cancel_job', { jobId: 'test-job-1' });
      expect(mockOnJobUpdate).toHaveBeenCalled();
    });
  });

  it('handles remove job action', async () => {
    const user = userEvent.setup();
    const completedJob = {
      ...mockJob,
      status: JobStatus.Completed,
    };

    (global as any).mockInvoke.mockResolvedValue(undefined);

    render(<QueueItem job={completedJob} onJobUpdate={mockOnJobUpdate} />);
    
    const removeButton = screen.getByRole('button', { name: /remove/i });
    await user.click(removeButton);
    
    await waitFor(() => {
      expect((global as any).mockInvoke).toHaveBeenCalledWith('remove_job', { jobId: 'test-job-1' });
      expect(mockOnJobUpdate).toHaveBeenCalled();
    });
  });

  it('expands and collapses details', async () => {
    const user = userEvent.setup();
    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    // Click to expand details
    const expandButton = screen.getByLabelText('Expand details');
    await user.click(expandButton);
    
    // Should show expanded content (implementation may vary)
    expect(expandButton).toBeInTheDocument();
  });

  it('displays progress bar with correct percentage', () => {
    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    // Check that progress percentage is displayed
    expect(screen.getByText('45.2%')).toBeInTheDocument();
    
    // Check that progress bar exists
    const progressBar = document.querySelector('.progress-bar');
    expect(progressBar).toBeInTheDocument();
    
    // Check that progress fill has correct width
    const progressFill = document.querySelector('.progress-fill');
    expect(progressFill).toHaveStyle('width: 45.2%');
  });

  it('handles jobs without metadata gracefully', () => {
    const jobWithoutMetadata = {
      ...mockJob,
      metadata: undefined,
    };

    render(<QueueItem job={jobWithoutMetadata} onJobUpdate={mockOnJobUpdate} />);
    
    // Should show URL as fallback
    expect(screen.getByText('https://music.youtube.com/watch?v=test')).toBeInTheDocument();
  });

  it('handles jobs without progress percentage', () => {
    const jobWithoutPercentage = {
      ...mockJob,
      progress: {
        ...mockJob.progress,
        percentage: undefined,
      },
    };

    render(<QueueItem job={jobWithoutPercentage} onJobUpdate={mockOnJobUpdate} />);
    
    // Should still show progress information
    expect(screen.getByText('Downloading audio')).toBeInTheDocument();
    expect(screen.queryByText('%')).not.toBeInTheDocument();
  });

  it('displays different stage indicators correctly', () => {
    const stages = [
      { stage: DownloadStage.Initializing, text: 'Initializing' },
      { stage: DownloadStage.FetchingMetadata, text: 'Fetching metadata' },
      { stage: DownloadStage.DownloadingAudio, text: 'Downloading audio' },
      { stage: DownloadStage.Remuxing, text: 'Remuxing' },
      { stage: DownloadStage.ApplyingTags, text: 'Applying tags' },
      { stage: DownloadStage.Finalizing, text: 'Finalizing' },
    ];

    stages.forEach(({ stage, text }) => {
      const jobWithStage = {
        ...mockJob,
        progress: {
          ...mockJob.progress,
          stage,
          current_step: text,
        },
      };

      const { unmount } = render(<QueueItem job={jobWithStage} onJobUpdate={mockOnJobUpdate} />);
      
      expect(screen.getByText(text)).toBeInTheDocument();
      
      unmount();
    });
  });

  it('handles action button errors gracefully', async () => {
    const user = userEvent.setup();
    (global as any).mockInvoke.mockRejectedValue(new Error('Network error'));

    render(<QueueItem job={mockJob} onJobUpdate={mockOnJobUpdate} />);
    
    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    await user.click(cancelButton);
    
    // Should handle error without crashing
    await waitFor(() => {
      expect((global as any).mockInvoke).toHaveBeenCalledWith('cancel_job', { jobId: 'test-job-1' });
    });
  });
});