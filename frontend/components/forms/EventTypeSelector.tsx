import React from 'react';

export type EventType =
    | 'HARVEST'
    | 'PROCESS'
    | 'PACKAGE'
    | 'SHIP'
    | 'RECEIVE'
    | 'QUALITY_CHECK'
    | 'TRANSFER';

export const EVENT_TYPES: { id: EventType; label: string; description: string; icon: string }[] = [
    { id: 'HARVEST', label: 'Harvest / Collect', description: 'Product emerged from origin', icon: '🌾' },
    { id: 'PROCESS', label: 'Process', description: 'Manufacturing or refining', icon: '⚙️' },
    { id: 'PACKAGE', label: 'Package', description: 'Placed into final packaging', icon: '📦' },
    { id: 'SHIP', label: 'Ship', description: 'In transit to next destination', icon: '🚚' },
    { id: 'RECEIVE', label: 'Receive', description: 'Arrived at destination', icon: '🏢' },
    { id: 'QUALITY_CHECK', label: 'Quality Check', description: 'Passed inspection', icon: '✅' },
    { id: 'TRANSFER', label: 'Transfer Ownership', description: 'Changed acting owner', icon: '🤝' },
];

interface EventTypeSelectorProps {
    value: EventType | '';
    onChange: (value: EventType) => void;
    error?: string;
}

export default function EventTypeSelector({ value, onChange, error }: EventTypeSelectorProps) {
    const groupId = React.useId();
    const errorId = error ? `${groupId}-error` : undefined;

    const handleKeyDown = (e: React.KeyboardEvent, typeId: EventType) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onChange(typeId);
        }
    };

    return (
        <div className="w-full">
            <div id={groupId} className="block text-sm font-medium text-gray-700 mb-3">Event Type *</div>
            <div
                role="radiogroup"
                aria-labelledby={groupId}
                aria-describedby={errorId}
                className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3"
            >
                {EVENT_TYPES.map((type) => (
                    <div
                        key={type.id}
                        role="radio"
                        aria-checked={value === type.id}
                        tabIndex={0}
                        onClick={() => onChange(type.id)}
                        onKeyDown={(e) => handleKeyDown(e, type.id)}
                        className={`cursor-pointer rounded-xl border p-4 transition-all flex flex-col items-start gap-2 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 ${value === type.id
                                ? 'border-indigo-600 bg-indigo-50 shadow-sm ring-1 ring-indigo-600'
                                : 'border-gray-200 bg-white hover:border-gray-300 hover:bg-gray-50 shadow-sm block'
                            }`}
                    >
                        <div className="flex items-center gap-2">
                            <span className="text-xl" role="img" aria-label={type.label}>{type.icon}</span>
                            <span className={`font-semibold text-sm ${value === type.id ? 'text-indigo-900' : 'text-gray-900'}`}>
                                {type.label}
                            </span>
                        </div>
                        <span className={`text-xs block w-full ${value === type.id ? 'text-indigo-700' : 'text-gray-500'}`}>
                            {type.description}
                        </span>
                    </div>
                ))}
            </div>
            {error && <p id={errorId} role="alert" className="mt-2 text-sm text-red-600">{error}</p>}
        </div>
    );
}
