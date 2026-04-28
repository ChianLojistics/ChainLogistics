"use client";

import { useState, useCallback } from "react";

/**
 * Loading state manager
 */
export interface LoadingState {
  isLoading: boolean;
  error: Error | null;
  progress: number;
  estimatedTime: number | null;
}

/**
 * Hook for managing async operation loading states
 */
export function useLoadingState(
  initialState: Partial<LoadingState> = {}
) {
  const [state, setState] = useState<LoadingState>({
    isLoading: false,
    error: null,
    progress: 0,
    estimatedTime: null,
    ...initialState,
  });

  const setLoading = useCallback((isLoading: boolean) => {
    setState((prev) => ({
      ...prev,
      isLoading,
      error: isLoading ? null : prev.error,
      progress: isLoading ? 0 : prev.progress,
    }));
  }, []);

  const setError = useCallback((error: Error | null) => {
    setState((prev) => ({
      ...prev,
      error,
      isLoading: false,
    }));
  }, []);

  const setProgress = useCallback((progress: number) => {
    setState((prev) => ({
      ...prev,
      progress: Math.min(Math.max(progress, 0), 100),
    }));
  }, []);

  const setEstimatedTime = useCallback((time: number | null) => {
    setState((prev) => ({
      ...prev,
      estimatedTime: time,
    }));
  }, []);

  const reset = useCallback(() => {
    setState({
      isLoading: false,
      error: null,
      progress: 0,
      estimatedTime: null,
    });
  }, []);

  return {
    ...state,
    setLoading,
    setError,
    setProgress,
    setEstimatedTime,
    reset,
  };
}

/**
 * Hook for async operations with automatic loading state management
 */
export function useAsync<T, E = Error>(
  asyncFunction: () => Promise<T>,
  immediate = true
) {
  const loading = useLoadingState();
  const [data, setData] = useState<T | null>(null);

  const execute = useCallback(async () => {
    loading.setLoading(true);
    loading.setError(null);
    try {
      const response = await asyncFunction();
      setData(response);
      loading.setLoading(false);
      return response;
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      loading.setError(err);
      loading.setLoading(false);
      throw err;
    }
  }, [asyncFunction, loading]);

  // Execute on mount if immediate is true
  if (immediate) {
    execute();
  }

  return { ...loading, data, execute };
}

/**
 * Hook for managing multi-step operations
 */
export interface Step {
  id: string;
  label: string;
  description?: string;
  duration?: number; // estimated duration in milliseconds
}

export interface MultiStepState {
  steps: Step[];
  currentStep: number;
  completedSteps: number[];
  isLoading: boolean;
  error: Error | null;
  progress: number;
}

export function useMultiStepOperation(steps: Step[]) {
  const [state, setState] = useState<MultiStepState>({
    steps,
    currentStep: 0,
    completedSteps: [],
    isLoading: false,
    error: null,
    progress: 0,
  });

  const goToNextStep = useCallback(() => {
    setState((prev) => ({
      ...prev,
      currentStep: Math.min(prev.currentStep + 1, prev.steps.length - 1),
    }));
  }, []);

  const markStepComplete = useCallback(() => {
    setState((prev) => ({
      ...prev,
      completedSteps: [...new Set([...prev.completedSteps, prev.currentStep])],
      progress: Math.round(
        ((prev.completedSteps.length + 1) / prev.steps.length) * 100
      ),
    }));
    goToNextStep();
  }, [goToNextStep]);

  const setStepError = useCallback((error: Error) => {
    setState((prev) => ({
      ...prev,
      error,
    }));
  }, []);

  const reset = useCallback(() => {
    setState({
      steps,
      currentStep: 0,
      completedSteps: [],
      isLoading: false,
      error: null,
      progress: 0,
    });
  }, [steps]);

  return {
    ...state,
    goToNextStep,
    markStepComplete,
    setStepError,
    reset,
  };
}
