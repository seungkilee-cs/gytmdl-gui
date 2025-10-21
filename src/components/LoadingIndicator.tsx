import React from 'react';
import { useLoadingState } from '../services/loadingState';
import './LoadingIndicator.css';

interface LoadingIndicatorProps {
  size?: 'small' | 'medium' | 'large';
  overlay?: boolean;
  message?: string;
}

const LoadingSpinner: React.FC<{ size: string }> = ({ size }) => (
  <div className={`loading-spinner loading-spinner-${size}`}>
    <div className="spinner-circle"></div>
  </div>
);

const LoadingIndicator: React.FC<LoadingIndicatorProps> = ({ 
  size = 'medium', 
  overlay = false, 
  message 
}) => {
  const { isAnyLoading } = useLoadingState();

  if (!isAnyLoading()) {
    return null;
  }

  const content = (
    <div className={`loading-indicator ${overlay ? 'loading-overlay' : ''}`}>
      <div className="loading-content">
        <LoadingSpinner size={size} />
        {message && <div className="loading-message">{message}</div>}
      </div>
    </div>
  );

  return content;
};

// Specific loading indicators for different operations
export const QueueLoadingIndicator: React.FC = () => {
  const { isLoading } = useLoadingState();
  
  if (!isLoading('queue-refresh') && !isLoading('queue-add') && !isLoading('queue-action')) {
    return null;
  }

  let message = 'Loading...';
  if (isLoading('queue-add')) message = 'Adding job to queue...';
  else if (isLoading('queue-action')) message = 'Processing queue action...';
  else if (isLoading('queue-refresh')) message = 'Refreshing queue...';

  return <LoadingIndicator size="small" message={message} />;
};

export const ConfigLoadingIndicator: React.FC = () => {
  const { isLoading } = useLoadingState();
  
  if (!isLoading('config-load') && !isLoading('config-save') && !isLoading('config-validate')) {
    return null;
  }

  let message = 'Loading...';
  if (isLoading('config-save')) message = 'Saving configuration...';
  else if (isLoading('config-validate')) message = 'Validating configuration...';
  else if (isLoading('config-load')) message = 'Loading configuration...';

  return <LoadingIndicator size="small" message={message} />;
};

export const CookieLoadingIndicator: React.FC = () => {
  const { isLoading } = useLoadingState();
  
  if (!isLoading('cookie-import') && !isLoading('cookie-validate') && !isLoading('cookie-clear')) {
    return null;
  }

  let message = 'Loading...';
  if (isLoading('cookie-import')) message = 'Importing cookies...';
  else if (isLoading('cookie-validate')) message = 'Validating cookies...';
  else if (isLoading('cookie-clear')) message = 'Clearing cookies...';

  return <LoadingIndicator size="small" message={message} />;
};

// Global loading overlay for full-screen operations
export const GlobalLoadingOverlay: React.FC = () => {
  const { isLoading } = useLoadingState();
  
  const isGlobalLoading = isLoading('app-init') || isLoading('app-update');
  
  if (!isGlobalLoading) {
    return null;
  }

  let message = 'Loading...';
  if (isLoading('app-init')) message = 'Initializing application...';
  else if (isLoading('app-update')) message = 'Updating application...';

  return (
    <LoadingIndicator 
      size="large" 
      overlay={true} 
      message={message} 
    />
  );
};

export default LoadingIndicator;