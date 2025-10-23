import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach } from 'vitest';
import ConfigEditor from '../components/ConfigEditor';
import { DownloadMode, CoverFormat } from '../types';

const mockConfig = {
  output_path: '/home/user/Music',
  temp_path: '/tmp/gytmdl',
  cookies_path: '/home/user/cookies.txt',
  itag: '141',
  download_mode: DownloadMode.Audio,
  concurrent_limit: 3,
  cover_size: 500,
  cover_format: CoverFormat.Jpg,
  cover_quality: 90,
  template_folder: '{artist}/{album}',
  template_file: '{track:02d} - {title}',
  template_date: '%Y-%m-%d',
  po_token: 'test_token',
  exclude_tags: undefined,
  truncate: undefined,
  save_cover: true,
  overwrite: false,
  no_synced_lyrics: false,
};

describe('ConfigEditor Component', () => {
  beforeEach(() => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'get_config') {
        return Promise.resolve(mockConfig);
      }
      if (command === 'update_config') {
        return Promise.resolve();
      }
      return Promise.resolve();
    });
  });

  it('renders configuration header and form', async () => {
    render(<ConfigEditor />);
    
    // Wait for configuration to load (initially shows loading)
    await waitFor(() => {
      expect(screen.getByText('Configuration')).toBeInTheDocument();
    });
    
    await waitFor(() => {
      expect(screen.getByText('Path Settings')).toBeInTheDocument();
      expect(screen.getByText('Download Settings')).toBeInTheDocument();
      expect(screen.getByText('Cover Art Settings')).toBeInTheDocument();
      expect(screen.getByText('File Templates')).toBeInTheDocument();
      expect(screen.getByText('Advanced Options')).toBeInTheDocument();
    });
  });

  it('loads and displays configuration values', async () => {
    render(<ConfigEditor />);
    
    await waitFor(() => {
      expect(screen.getByDisplayValue('/home/user/Music')).toBeInTheDocument();
      expect(screen.getByDisplayValue('/tmp/gytmdl')).toBeInTheDocument();
      expect(screen.getByDisplayValue('3')).toBeInTheDocument();
      expect(screen.getByDisplayValue('{artist}/{album}')).toBeInTheDocument();
    });
  });

  it('shows loading state initially', () => {
    render(<ConfigEditor />);
    
    expect(screen.getByText('Loading configuration...')).toBeInTheDocument();
  });

  it('enables save button when changes are made', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('/home/user/Music')).toBeInTheDocument();
    });
    
    const saveButton = screen.getByRole('button', { name: /save configuration/i });
    expect(saveButton).toBeDisabled(); // Initially disabled (no changes)
    
    // Make a change
    const outputPathInput = screen.getByDisplayValue('/home/user/Music');
    await user.clear(outputPathInput);
    await user.type(outputPathInput, '/new/path');
    
    await waitFor(() => {
      expect(saveButton).not.toBeDisabled();
    });
  });

  it('validates required fields', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('/home/user/Music')).toBeInTheDocument();
    });
    
    // Clear required field
    const outputPathInput = screen.getByDisplayValue('/home/user/Music');
    await user.clear(outputPathInput);
    
    const saveButton = screen.getByRole('button', { name: /save configuration/i });
    await user.click(saveButton);
    
    await waitFor(() => {
      expect(screen.getByText('Output path is required')).toBeInTheDocument();
    });
  });

  it('validates concurrent limit range', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('3')).toBeInTheDocument();
    });
    
    // Set invalid concurrent limit
    const concurrentInput = screen.getByDisplayValue('3');
    await user.clear(concurrentInput);
    await user.type(concurrentInput, '15');
    
    const saveButton = screen.getByRole('button', { name: /save configuration/i });
    await user.click(saveButton);
    
    await waitFor(() => {
      expect(screen.getByText('Concurrent limit must be between 1 and 10')).toBeInTheDocument();
    });
  });

  it('validates cover size range', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('500')).toBeInTheDocument();
    });
    
    // Set invalid cover size
    const coverSizeInput = screen.getByDisplayValue('500');
    await user.clear(coverSizeInput);
    await user.type(coverSizeInput, '50');
    
    const saveButton = screen.getByRole('button', { name: /save configuration/i });
    await user.click(saveButton);
    
    await waitFor(() => {
      expect(screen.getByText('Cover size must be between 100 and 2000 pixels')).toBeInTheDocument();
    });
  });

  it('saves configuration when valid', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('/home/user/Music')).toBeInTheDocument();
    });
    
    // Make a valid change
    const outputPathInput = screen.getByDisplayValue('/home/user/Music');
    await user.clear(outputPathInput);
    await user.type(outputPathInput, '/new/valid/path');
    
    const saveButton = screen.getByRole('button', { name: /save configuration/i });
    await user.click(saveButton);
    
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('update_config', {
        config: expect.objectContaining({
          output_path: '/new/valid/path',
        }),
      });
    });
  });

  it('shows success message after saving', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('/home/user/Music')).toBeInTheDocument();
    });
    
    // Make a change and save
    const outputPathInput = screen.getByDisplayValue('/home/user/Music');
    await user.clear(outputPathInput);
    await user.type(outputPathInput, '/new/path');
    
    const saveButton = screen.getByRole('button', { name: /save configuration/i });
    await user.click(saveButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Configuration saved successfully/)).toBeInTheDocument();
    });
  });

  it('resets changes when reset button is clicked', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('/home/user/Music')).toBeInTheDocument();
    });
    
    // Make a change
    const outputPathInput = screen.getByDisplayValue('/home/user/Music');
    await user.clear(outputPathInput);
    await user.type(outputPathInput, '/changed/path');
    
    // Reset changes
    const resetButton = screen.getByRole('button', { name: /reset changes/i });
    await user.click(resetButton);
    
    await waitFor(() => {
      expect(screen.getByDisplayValue('/home/user/Music')).toBeInTheDocument();
    });
  });

  it('handles audio quality selection', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('AAC 256kbps (Best Quality)')).toBeInTheDocument();
    });
    
    // Change audio quality
    const itagSelect = screen.getByDisplayValue('AAC 256kbps (Best Quality)');
    await user.selectOptions(itagSelect, '140');
    
    expect(screen.getByDisplayValue('AAC 128kbps (Standard Quality)')).toBeInTheDocument();
  });

  it('handles download mode selection', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('Audio Only')).toBeInTheDocument();
    });
    
    // Change download mode
    const modeSelect = screen.getByDisplayValue('Audio Only');
    await user.selectOptions(modeSelect, DownloadMode.Video);
    
    expect(screen.getByDisplayValue('Video Only')).toBeInTheDocument();
  });

  it('handles checkbox toggles', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByLabelText(/save cover art as separate file/i)).toBeChecked();
    });
    
    // Toggle checkbox
    const saveCoverCheckbox = screen.getByLabelText(/save cover art as separate file/i);
    await user.click(saveCoverCheckbox);
    
    expect(saveCoverCheckbox).not.toBeChecked();
  });

  it('handles template field updates', async () => {
    const user = userEvent.setup();
    render(<ConfigEditor />);
    
    // Wait for config to load
    await waitFor(() => {
      expect(screen.getByDisplayValue('{artist}/{album}')).toBeInTheDocument();
    });
    
    // Update template - use a simpler approach
    const folderTemplateInput = screen.getByDisplayValue('{artist}/{album}');
    
    // Clear and type new value
    await user.clear(folderTemplateInput);
    await user.type(folderTemplateInput, 'test-template');
    
    // Check that the input value was updated
    expect(folderTemplateInput).toHaveValue('test-template');
  });
});