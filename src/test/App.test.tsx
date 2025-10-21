import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import App from '../App';

describe('App Component', () => {
  it('renders the main application layout', () => {
    render(<App />);
    
    // Check header elements
    expect(screen.getByText('gytmdl GUI')).toBeInTheDocument();
    expect(screen.getByText('YouTube Music Downloader')).toBeInTheDocument();
    
    // Check navigation buttons using more specific selectors
    expect(screen.getByText('Queue')).toBeInTheDocument();
    expect(screen.getByText('Config')).toBeInTheDocument();
    expect(screen.getByText('Cookies')).toBeInTheDocument();
  });

  it('starts with queue view active', () => {
    render(<App />);
    
    const queueButton = screen.getByText('Queue').closest('button');
    expect(queueButton).toHaveClass('active');
  });

  it('switches between views when navigation buttons are clicked', () => {
    render(<App />);
    
    // Get navigation buttons using text content
    const queueButton = screen.getByText('Queue').closest('button');
    const configButton = screen.getByText('Config').closest('button');
    const cookiesButton = screen.getByText('Cookies').closest('button');
    
    expect(queueButton).toHaveClass('active');
    
    // Click config button
    fireEvent.click(configButton!);
    expect(configButton).toHaveClass('active');
    expect(queueButton).not.toHaveClass('active');
    
    // Click cookies button
    fireEvent.click(cookiesButton!);
    expect(cookiesButton).toHaveClass('active');
    expect(configButton).not.toHaveClass('active');
    
    // Click queue button again
    fireEvent.click(queueButton!);
    expect(queueButton).toHaveClass('active');
    expect(cookiesButton).not.toHaveClass('active');
  });

  it('renders the correct view content based on active view', async () => {
    render(<App />);
    
    // Queue view should be visible initially
    expect(screen.getByText('Download Queue')).toBeInTheDocument();
    
    // Switch to config view
    const configButton = screen.getByText('Config').closest('button');
    fireEvent.click(configButton!);
    
    // Config view shows loading state initially, which is expected behavior
    // Check that we're in the config view by looking for config-specific content
    expect(screen.getByText('Loading configuration...')).toBeInTheDocument();
    
    // Switch to cookies view
    const cookiesButton = screen.getByText('Cookies').closest('button');
    fireEvent.click(cookiesButton!);
    expect(screen.getByText('Cookie Manager')).toBeInTheDocument();
  });
});