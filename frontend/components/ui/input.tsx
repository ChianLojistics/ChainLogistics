import React from 'react';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    label?: string;
    error?: string;
    icon?: React.ReactNode;
}

export function Input({ label, error, icon, className = '', ...props }: InputProps) {
    return (
        <div className="w-full space-y-1.5">
            {label && <label className="block text-sm font-medium text-gray-700">{label}</label>}
            <div className="relative">
                {icon && (
                    <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none text-gray-400">
                        {icon}
                    </div>
                )}
                <input
                    className={`
            block w-full rounded-lg border border-gray-300 bg-white py-2 px-3 text-gray-900 
            transition-colors placeholder:text-gray-400
            focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500
            ${icon ? 'pl-10' : ''}
            ${error ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : ''}
            ${className}
          `}
                    {...props}
                />
            </div>
            {error && <p className="text-sm text-red-600">{error}</p>}
        </div>
    );
}
