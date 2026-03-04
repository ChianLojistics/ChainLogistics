"use client";

import React from 'react';
import { Button } from '@/components/ui/button';
import { toast } from '@/lib/toast/store';

export default function ToastDemo() {
  const showSuccessToast = () => {
    toast.success('Operation completed successfully!', {
      title: 'Success',
      duration: 5000,
    });
  };

  const showErrorToast = () => {
    toast.error('Something went wrong. Please try again.', {
      title: 'Error',
      duration: 7000,
    });
  };

  const showInfoToast = () => {
    toast.info('Here is some useful information for you.', {
      title: 'Information',
      duration: 4000,
    });
  };

  const showWarningToast = () => {
    toast.warning('Please review this important warning.', {
      title: 'Warning',
      duration: 6000,
    });
  };

  const showToastWithAction = () => {
    toast.success('File uploaded successfully!', {
      title: 'Upload Complete',
      action: {
        label: 'View File',
        onClick: () => {
          alert('Viewing file...');
        },
      },
      duration: 8000,
    });
  };

  const showMultipleToasts = () => {
    toast.success('First toast');
    setTimeout(() => toast.info('Second toast'), 500);
    setTimeout(() => toast.warning('Third toast'), 1000);
    setTimeout(() => toast.error('Fourth toast'), 1500);
  };

  const clearAllToasts = () => {
    toast.clear();
  };

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-2xl mx-auto">
        <h1 className="text-3xl font-bold text-gray-900 mb-8">
          Toast Notification System Demo
        </h1>
        
        <div className="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 className="text-xl font-semibold text-gray-800 mb-4">
            Basic Toast Types
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <Button onClick={showSuccessToast} variant="default">
              Success
            </Button>
            <Button onClick={showErrorToast} variant="destructive">
              Error
            </Button>
            <Button onClick={showInfoToast} variant="outline">
              Info
            </Button>
            <Button onClick={showWarningToast} variant="secondary">
              Warning
            </Button>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 className="text-xl font-semibold text-gray-800 mb-4">
            Advanced Features
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Button onClick={showToastWithAction} variant="outline">
              Toast with Action
            </Button>
            <Button onClick={showMultipleToasts} variant="outline">
              Show Multiple Toasts
            </Button>
            <Button onClick={clearAllToasts} variant="outline">
              Clear All Toasts
            </Button>
          </div>
        </div>

        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h3 className="font-semibold text-blue-900 mb-2">Usage Instructions:</h3>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>• Click the buttons above to see different toast types</li>
            <li>• Toasts auto-dismiss after the specified duration</li>
            <li>• Hover over toasts to see the dismiss button</li>
            <li>• Multiple toasts stack automatically</li>
            <li>• Click the X button to dismiss manually</li>
            <li>• Toasts are announced to screen readers for accessibility</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
