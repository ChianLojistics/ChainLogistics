"use client";

import { Check } from "lucide-react";

type Step = {
    id: number;
    name: string;
};

interface FormStepIndicatorProps {
    steps: Step[];
    currentStep: number;
}

export function FormStepIndicator({ steps, currentStep }: FormStepIndicatorProps) {
    return (
        <nav aria-label="Progress" className="mb-8">
            <ol role="list" className="flex items-center justify-center space-x-8">
                {steps.map((step, stepIdx) => (
                    <li key={step.name} className="relative flex items-center">
                        <div className="flex flex-col items-center">
                            <div
                                className={`
                  flex h-10 w-10 items-center justify-center rounded-full border-2 transition-all
                  ${step.id < currentStep
                                        ? "bg-blue-600 border-blue-600"
                                        : step.id === currentStep
                                            ? "border-blue-600 bg-white"
                                            : "border-gray-300 bg-white"}
                `}
                            >
                                {step.id < currentStep ? (
                                    <Check className="h-6 w-6 text-white" aria-hidden="true" />
                                ) : (
                                    <span className={`text-sm font-semibold ${step.id === currentStep ? "text-blue-600" : "text-gray-500"}`}>
                                        {step.id}
                                    </span>
                                )}
                            </div>
                            <span className={`mt-2 text-xs font-medium ${step.id <= currentStep ? "text-blue-600" : "text-gray-500"}`}>
                                {step.name}
                            </span>
                        </div>
                        {stepIdx !== steps.length - 1 && (
                            <div
                                className={`ml-4 h-0.5 w-12 bg-gray-200 transition-all ${step.id < currentStep ? "bg-blue-600" : "bg-gray-200"}`}
                                style={{ position: "absolute", top: "20px", left: "40px" }}
                            />
                        )}
                    </li>
                ))}
            </ol>
        </nav>
    );
}
