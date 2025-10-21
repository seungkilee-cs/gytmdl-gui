import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach } from 'vitest';
import QueueView from '../components/QueueView';
import { JobStatus, DownloadStage } from '../types';

const mockJobs = [
  {
    id: '1',
    url: 'https://music.youtube.com/watch?v=test1',
    status: JobStatus.Queued,
    progress: {
      stage: DownloadStage.Initializing,
      current_step: 'Waiting in queue',
      percentage: 0,
    },
    created_at: '2024-01-01T00:00:00Z',
    metadata: {
      title: 'Test Song 1',
      artist: 'Test Artist',
      album: 'Test Album',
    },
  },
  {
    id: '2',
    url: 'https://music.youtube.com/watch?v=test2',
    status: JobStatus.Downloading,
    progress: {
      stage: DownloadStage.DownloadingAudio,
      current_step: 'Downloading audio',
      percentage: 45.2,
    },
    created_at: '2024-01-01T00:01:00Z',
    metadata: {
      title: 'Test Song 2',
      artist: 'Test Artist 2',
    },
  },
  {
    id: '3',
    url: 'https://music.youtube.com/watch?v=test3',
    status: JobStatus.Completed,
    progress: {
      stage: DownloadStage.Completed,
      current_step: 'Download completed',
      percentage: 100,
    },
    created_at: '2024-01-01T00:02:00Z',
    completed_at: '2024-01-01T00:05:00Z',
    metadata: {
      title: 'Test Song 3',
      artist: 'Test Artist 3',
    },
  },
];

describe('QueueView Component', () => {
  beforeEach(() => {
    // Mock the get_queue response
    (global as any).mockInvoke.mockImplementation((command: string) => {
      if (command === 'get_queue') {
        return Promise.resolve({
          jobs: mockJobs,
          is_paused: false,
        });
      }
      return Promise.resolve();
    });
  });

  it('renders queue header and statistics', async () => {
    render(<QueueView />);
    
    expect(screen.getByText('Download Queue')).toBeInTheDocument();
    
    // Wait for jobs to load and check statistics
    await waitFor(() => {
      // Check that jobs are loaded by looking for job titles
      expect(screen.getByText('Test Song 1')).toBeInTheDocument();
    });
    
    // Check statistics using more specific selectors
    const statItems = document.querySelectorAll('.stat-item');
    expect(statItems[0].querySelector('.stat-value')).toHaveTextContent('3'); // Total
    expect(statItems[1].querySelector('.stat-value')).toHaveTextContent('1'); // Queued
    expect(statItems[2].querySelector('.stat-value')).toHaveTextContent('1'); // Downloading
    expect(statItems[3].querySelector('.stat-value')).toHaveTextContent('1'); // Completed
  });

  it('renders URL input form', () => {
    render(<QueueView />);
    
    const urlInput = screen.getByPlaceholderText(/enter youtube music url/i);
    const addButton = screen.getByRole('button', { name: /add/i });
    
    expect(urlInput).toBeInTheDocument();
    expect(addButton).toBeInTheDocument();
    expect(addButton).toBeDisabled(); // Should be disabled when input is empty
  });

  it('enables add button when URL is entered', async () => {
    const user = userEvent.setup();
    render(<QueueView />);
    
    const urlInput = screen.getByPlaceholderText(/enter youtube music url/i);
    const addButton = screen.getByRole('button', { name: /add/i });
    
    await user.type(urlInput, 'https://music.youtube.com/watch?v=test');
    
    expect(addButton).not.toBeDisabled();
  });

  it('validates YouTube Music URLs', async () => {
    const user = userEvent.setup();
    render(<QueueView />);
    
    const urlInput = screen.getByPlaceholderText(/enter youtube music url/i);
    const addButton = screen.getByRole('button', { name: /add/i });
    
    // Test invalid URL
    await user.type(urlInput, 'https://example.com/invalid');
    await user.click(addButton);
    
    await waitFor(() => {
      expect(screen.getByText(/please enter a valid youtube music url/i)).toBeInTheDocument();
    });
  });

  it('calls add_to_queue when valid URL is submitted', async () => {
    const user = userEvent.setup();
    render(<QueueView />);
    
    const urlInput = screen.getByPlaceholderText(/enter youtube music url/i);
    const addButton = screen.getByRole('button', { name: /add/i });
    
    const testUrl = 'https://music.youtube.com/watch?v=test';
    await user.type(urlInput, testUrl);
    await user.click(addButton);
    
    await waitFor(() => {
      expect((global as any).mockInvoke).toHaveBeenCalledWith('add_to_queue', { url: testUrl });
    });
  });

  it('renders job list with correct job information', async () => {
    render(<QueueView />);
    
    // Wait for jobs to load
    await waitFor(() => {
      expect(screen.getByText('Test Song 1')).toBeInTheDocument();
      expect(screen.getByText('Test Song 2')).toBeInTheDocument();
      expect(screen.getByText('Test Song 3')).toBeInTheDocument();
    });
  });

  it('shows empty state when no jobs exist', async () => {
    (global as any).mockInvoke.mockImplementation((command: string) => {
      if (command === 'get_queue') {
        return Promise.resolve({
          jobs: [],
          is_paused: false,
        });
      }
      return Promise.resolve();
    });

    render(<QueueView />);
    
    await waitFor(() => {
      expect(screen.getByText('No jobs in queue')).toBeInTheDocument();
      expect(screen.getByText('Add a YouTube Music URL to get started')).toBeInTheDocument();
    });
  });

  it('handles pause/resume queue functionality', async () => {
    const user = userEvent.setup();
    render(<QueueView />);
    
    // Wait for component to load
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /pause queue/i })).toBeInTheDocument();
    });
    
    const pauseButton = screen.getByRole('button', { name: /pause queue/i });
    await user.click(pauseButton);
    
    expect((global as any).mockInvoke).toHaveBeenCalledWith('pause_queue');
  });

  it('handles clear completed jobs functionality', async () => {
    const user = userEvent.setup();
    render(<QueueView />);
    
    // Wait for jobs to load
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /clear completed/i })).toBeInTheDocument();
    });
    
    const clearButton = screen.getByRole('button', { name: /clear completed/i });
    await user.click(clearButton);
    
    await waitFor(() => {
      expect((global as any).mockInvoke).toHaveBeenCalledWith('remove_job', { jobId: '3' });
    });
  });

  it('filters jobs by status', async () => {
    const user = userEvent.setup();
    render(<QueueView />);
    
    // Wait for jobs to load
    await waitFor(() => {
      expect(screen.getByText('Test Song 1')).toBeInTheDocument();
    });
    
    // Filter by completed jobs only
    const filterSelect = screen.getByDisplayValue('All Jobs');
    await user.selectOptions(filterSelect, 'completed');
    
    // Should only show completed job
    expect(screen.getByText('Test Song 3')).toBeInTheDocument();
    expect(screen.queryByText('Test Song 1')).not.toBeInTheDocument();
    expect(screen.queryByText('Test Song 2')).not.toBeInTheDocument();
  });

  it('handles drag and drop URL input', async () => {
    render(<QueueView />);
    
    const dropZone = screen.getByPlaceholderText(/enter youtube music url/i).closest('.url-input-container');
    
    const testUrl = 'https://music.youtube.com/watch?v=dragtest';
    const dropEvent = new Event('drop', { bubbles: true });
    Object.defineProperty(dropEvent, 'dataTransfer', {
      value: {
        getData: () => testUrl,
      },
    });
    
    fireEvent(dropZone!, dropEvent);
    
    await waitFor(() => {
      expect((global as any).mockInvoke).toHaveBeenCalledWith('add_to_queue', { url: testUrl });
    });
  });
});