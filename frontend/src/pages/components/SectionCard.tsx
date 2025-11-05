import React from 'react';

interface SectionCardProps {
    title: string;
    children: React.ReactNode;
    actions?: React.ReactNode;
    className?: string; // Add className prop
}

const SectionCard: React.FC<SectionCardProps> = ({ title, children, actions, className = '' }) => {
    return (
        <div className={`bg-white rounded-lg shadow-sm p-4 mb-4 transition-shadow duration-200 ease-in-out hover:shadow-sm ${className}`}>
            <div className="flex items-center justify-between mb-3 pb-2 border-b border-gray-100">
                <h2 className="text-base font-semibold text-gray-900">{title}</h2>
                {actions}
            </div>
            <div className="space-y-3">
                {children}
            </div>
        </div>
    );
};

export default SectionCard;