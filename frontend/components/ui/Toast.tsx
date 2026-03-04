"use client";

import React from 'react';
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-react';
import type { Toast } from '@/lib/toast/store';
import { cn } from '@/lib/utils';
import { Button } from './button';

interface ToastComponentProps {
  toast: Toast;
  onDismiss: (id: string) => void;
}

const toastIcons = {
  success: CheckCircle,
  error: AlertCircle,
  info: Info,
  warning: AlertTriangle,
};

const toastVariants = {
  success: 'bg-green-50 border-green-200 text-green-800',
  error: 'bg-red-50 border-red-200 text-red-800',
  info: 'bg-blue-50 border-blue-200 text-blue-800',
  warning: 'bg-yellow-50 border-yellow-200 text-yellow-800',
};

const iconVariants = {
  success: 'text-green-600',
  error: 'text-red-600',
  info: 'text-blue-600',
  warning: 'text-yellow-600',
};

export const ToastComponent: React.FC<ToastComponentProps> = ({ toast, onDismiss }) => {
  const Icon = toastIcons[toast.type];
  
  const handleDismiss = () => {
    onDismiss(toast.id);
  };

  const handleAction = () => {
    if (toast.action?.onClick) {
      toast.action.onClick();
    }
  };

  return (
    <div
      role="alert"
      aria-live="polite"
      aria-atomic="true"
      className={cn(
        'group relative flex w-full max-w-md items-start gap-3 rounded-lg border p-4 shadow-lg transition-all duration-300 ease-in-out',
        'animate-in slide-in-from-right-full fade-in-0',
        'hover:shadow-xl',
        toastVariants[toast.type]
      )}
    >
      {/* Icon */}
      <div className={cn('flex-shrink-0', iconVariants[toast.type])}>
        <Icon className="h-5 w-5" />
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        {toast.title && (
          <h4 className="font-semibold text-sm mb-1">{toast.title}</h4>
        )}
        <p className="text-sm leading-relaxed">{toast.message}</p>
        
        {/* Action button */}
        {toast.action && (
          <div className="mt-3">
            <Button
              variant="outline"
              size="sm"
              onClick={handleAction}
              className="text-xs"
            >
              {toast.action.label}
            </Button>
          </div>
        )}
      </div>

      {/* Dismiss button */}
      <Button
        variant="ghost"
        size="icon"
        onClick={handleDismiss}
        className={cn(
          'flex-shrink-0 h-6 w-6 rounded-full opacity-0 transition-opacity',
          'group-hover:opacity-100',
          'hover:bg-black/10 focus:opacity-100',
          iconVariants[toast.type]
        )}
        aria-label="Dismiss notification"
      >
        <X className="h-4 w-4" />
      </Button>
    </div>
  );
};
